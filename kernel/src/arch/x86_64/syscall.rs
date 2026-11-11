use x86_64::registers::model_specific::{Efer, EferFlags, Star, LStar, SFMask};
use x86_64::VirtAddr;
use x86_64::structures::gdt::SegmentSelector;

#[repr(C)]
pub struct SyscallFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,

    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,

    pub user_rip: u64,
    pub user_rsp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Syscall {
    Exit = 0,
    Yield = 1,
    Send = 2,
    Recv = 3,
    Register = 4,
    Find = 5,
    SpawnProcess = 6,
    KillProcess = 7,
    WaitProcess = 8,
    Snapshot = 9,
    GetHistory = 10,
    GetPciDevices = 11,
    UpdateDeviceState = 12,
    OpenDevice = 13,
    ReadDevice = 14,
    WriteDevice = 15,
    Ioctl = 16,
    RegisterDriver = 17,
    UpdateDriverState = 18,
    Open = 19,
    Read = 20,
    Write = 21,
    Close = 22,
    Mkdir = 23,
    Mount = 24,
}

pub fn init(kernel_code: SegmentSelector, kernel_data: SegmentSelector, user_code: SegmentSelector, user_data: SegmentSelector) {
    extern "C" {
        fn syscall_entry();
    }
    unsafe {
        // Enable System Call Extensions (SCE)
        Efer::update(|flags| flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));

        // Setup STAR: CS/SS for syscall (kernel) and sysret (user)
        Star::write(
            user_code,    // sysret CS/SS (User code/data are usually adjacent in GDT)
            user_data,
            kernel_code,  // syscall CS/SS
            kernel_data,
        ).unwrap();

        // Setup LSTAR: Target RIP for syscall
        LStar::write(VirtAddr::new(syscall_entry as u64));

        // Setup SFMASK: RFLAGS mask to apply (clear IF to disable interrupts during syscall handler entry)
        SFMask::write(x86_64::registers::rflags::RFlags::INTERRUPT_FLAG);
    }
}

core::arch::global_asm!(
    r#"
    .global syscall_entry
    syscall_entry:
        // 1. Swap GS base to get CpuLocal
        swapgs

        // 2. Save user RSP to a scratch register
        mov r10, rsp // Actually, usually we can just read kernel_stack_top from GS

        // 3. Load kernel stack
        // Offset of kernel_stack_top in CpuLocal is 24 (u32 + pad + u64 + u64 = 4+4+8+8 = 24)
        mov rsp, gs:[24]

        // 4. Push SyscallFrame
        push r10 // user_rsp
        push rcx // user_rip (saved by syscall instruction)
        push rax
        push rcx // rcx general reg
        push rdx
        push rbx
        push rbp
        push rsi
        push rdi
        push r8
        push r9
        push r10 // r10 general reg
        push r11 // RFLAGS (saved by syscall instruction)
        push r12
        push r13
        push r14
        push r15

        // Pass pointer to SyscallFrame as first argument (RDI)
        mov rdi, rsp

        // 5. Call Rust Dispatcher
        call syscall_dispatch

        // 6. Restore SyscallFrame
        pop r15
        pop r14
        pop r13
        pop r12
        pop r11
        pop r10
        pop r9
        pop r8
        pop rdi
        pop rsi
        pop rbp
        pop rbx
        pop rdx
        pop rcx
        pop rax

        // Pop user_rip (rcx) and user_rsp (r10)
        pop rcx
        pop r10

        // Restore user stack
        mov rsp, r10

        // Restore GS base
        swapgs

        // Return to user space
        sysretq
    "#
);

#[no_mangle]
pub extern "C" fn syscall_dispatch(frame: &mut SyscallFrame) {
    crate::println!("SYSCALL {} called!", frame.rax);
    // RAX = syscall number, RDI = arg1, RSI = arg2, RDX = arg3

    if frame.rax == Syscall::Exit as u64 {
        let code = frame.rdi as i32;
        crate::println!("Process EXITED with code {}", code);
        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        if !cpu_local.current_process.is_null() {
            let process = unsafe { &*cpu_local.current_process };
            *process.state.lock() = crate::object::process::ProcessState::Zombie;
            *process.exit_info.lock() = Some(crate::object::process::ProcessExit {
                state: crate::object::process::ProcessState::Zombie,
                code,
            });
        }
        if !cpu_local.current_thread.is_null() {
            // Unschedule it. Yielding will put it back, but the scheduler will ignore it if process is Zombie!
            crate::task::scheduler::yield_current();
        }
    } else if frame.rax == Syscall::Yield as u64 {
        crate::task::scheduler::yield_current();
    } else if frame.rax == Syscall::Send as u64 {
        // arg1 = handle, arg2 = user_ptr, arg3 = size
        let handle = crate::object::handle::Handle(frame.rdi);
        let user_ptr = frame.rsi as *const u8;
        let size = frame.rdx as usize;
        
        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        if !cpu_local.current_process.is_null() {
            let process = unsafe { &*cpu_local.current_process };
            if let Some(entry) = process.handle_table.get(handle) {
                if entry.rights.contains(crate::object::handle::HandleRights::WRITE) {
                    if let Some(channel) = entry.object.as_any().downcast_ref::<crate::object::ipc::ChannelObject>() {
                        if size <= crate::object::ipc::IPC_MAX_MESSAGE_SIZE {
                            let mut msg = crate::object::ipc::Message {
                                len: size as u16,
                                data: [0; crate::object::ipc::IPC_MAX_MESSAGE_SIZE],
                            };
                            if crate::memory::user_copy::copy_from_user(user_ptr, &mut msg.data[..size]) {
                                channel.send(msg);
                                frame.rax = 0; // Success
                            } else {
                                frame.rax = 14; // EFAULT
                            }
                        } else {
                            frame.rax = 22; // EINVAL
                        }
                    } else {
                        frame.rax = 9; // EBADF (not a channel)
                    }
                } else {
                    frame.rax = 13; // EACCES
                }
            } else {
                frame.rax = 9; // EBADF
            }
        } else {
            frame.rax = 1; // EPERM
        }
    } else if frame.rax == Syscall::Recv as u64 {
        // arg1 = handle, arg2 = user_ptr, arg3 = max_size
        let handle = crate::object::handle::Handle(frame.rdi);
        let user_ptr = frame.rsi as *mut u8;
        let max_size = frame.rdx as usize;

        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        if !cpu_local.current_process.is_null() {
            let process = unsafe { &*cpu_local.current_process };
            if let Some(entry) = process.handle_table.get(handle) {
                if entry.rights.contains(crate::object::handle::HandleRights::READ) {
                    if let Some(channel) = entry.object.as_any().downcast_ref::<crate::object::ipc::ChannelObject>() {
                        if let Some(msg) = channel.try_recv() {
                            let copy_size = core::cmp::min(msg.len as usize, max_size);
                            if crate::memory::user_copy::copy_to_user(&msg.data[..copy_size], user_ptr) {
                                frame.rax = copy_size as u64; // Bytes read
                            } else {
                                // We dropped a message because of EFAULT! In a real OS we might put it back or leave it.
                                frame.rax = 14; // EFAULT
                            }
                        } else {
                            frame.rax = 11; // EAGAIN / WouldBlock
                        }
                    } else {
                        frame.rax = 9; // EBADF
                    }
                } else {
                    frame.rax = 13; // EACCES
                }
            } else {
                frame.rax = 9; // EBADF
            }
        } else {
            frame.rax = 1; // EPERM
        }
    } else if frame.rax == Syscall::Register as u64 {
        let name_ptr = frame.rdi as *const u8;
        let name_len = frame.rsi as usize;
        if name_len > 64 { frame.rax = 22; return; } // EINVAL
        let mut name_buf = [0u8; 64];
        if crate::memory::user_copy::copy_from_user(name_ptr, &mut name_buf[..name_len]) {
            if let Ok(name_str) = core::str::from_utf8(&name_buf[..name_len]) {
                let mut registry = crate::object::ipc::REGISTRY.lock();
                if registry.contains_key(name_str) {
                    frame.rax = 17; // EEXIST
                } else {
                    let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
                    if !cpu_local.current_process.is_null() {
                        let process = unsafe { &*cpu_local.current_process };
                        let channel = alloc::sync::Arc::new(crate::object::ipc::ChannelObject::new(process.id));
                        let handle = process.handle_table.add(channel.clone(), crate::object::handle::HandleRights::READ);
                        
                        registry.insert(alloc::string::String::from(name_str), crate::object::ipc::ServiceRegistration {
                            pid: process.id,
                            channel: channel,
                            start_time: 0, // TODO: timer
                            restart_count: 0,
                            singleton: true, // For now all registered are singleton
                        });
                        frame.rax = handle.0;
                    } else { frame.rax = 1; }
                }
            } else { frame.rax = 22; }
        } else { frame.rax = 14; }
    } else if frame.rax == Syscall::Find as u64 {
        let name_ptr = frame.rdi as *const u8;
        let name_len = frame.rsi as usize;
        if name_len > 64 { frame.rax = 22; return; }
        let mut name_buf = [0u8; 64];
        if crate::memory::user_copy::copy_from_user(name_ptr, &mut name_buf[..name_len]) {
            if let Ok(name_str) = core::str::from_utf8(&name_buf[..name_len]) {
                let registry = crate::object::ipc::REGISTRY.lock();
                if let Some(reg) = registry.get(name_str) {
                    let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
                    if !cpu_local.current_process.is_null() {
                        let process = unsafe { &*cpu_local.current_process };
                        let handle = process.handle_table.add(reg.channel.clone(), crate::object::handle::HandleRights::WRITE);
                        frame.rax = handle.0;
                    } else { frame.rax = 1; }
                } else {
                    frame.rax = 2; // ENOENT
                }
            } else { frame.rax = 22; }
        } else { frame.rax = 14; }
    } else if frame.rax == Syscall::SpawnProcess as u64 {
        let name_ptr = frame.rdi as *const u8;
        let name_len = frame.rsi as usize;
        if name_len > 64 { frame.rax = 22; return; }
        let mut name_buf = [0u8; 64];
        if crate::memory::user_copy::copy_from_user(name_ptr, &mut name_buf[..name_len]) {
            if let Ok(name_str) = core::str::from_utf8(&name_buf[..name_len]) {
                if let Some(data) = unsafe { crate::memory::tar::INITRD_DATA } {
                    let archive = crate::memory::tar::TarArchive::new(data);
                    if let Some(elf_data) = archive.find(name_str) {
                        if let Some(mut new_process) = crate::object::process::ProcessObject::new(false, name_str) {
                            if let Some(image) = crate::memory::elf::load_elf(elf_data, &mut new_process) {
                                let process_arc = alloc::sync::Arc::new(new_process);
                                let pid = process_arc.id.0;
                                let thread = alloc::sync::Arc::new(crate::object::thread::ThreadObject::new_user(image.entry_point, alloc::sync::Arc::downgrade(&process_arc)));
                                process_arc.add_thread(thread.clone());
                                crate::task::scheduler::add_task(thread);
                                
                                crate::task::scheduler::SCHEDULER.lock().add_process(process_arc);
                                frame.rax = pid;
                            } else { frame.rax = 8; } // ENOEXEC
                        } else { frame.rax = 12; } // ENOMEM
                    } else { frame.rax = 2; } // ENOENT
                } else { frame.rax = 2; }
            } else { frame.rax = 22; }
        } else { frame.rax = 14; }
    } else if frame.rax == Syscall::WaitProcess as u64 {
        let target_pid = frame.rdi as u64;
        let status_ptr = frame.rsi as *mut crate::object::process::ProcessExit;
        let mut sched = crate::task::scheduler::SCHEDULER.lock();
        if let Some(process) = sched.get_process(crate::object::process::ProcessId(target_pid)) {
            let state = *process.state.lock();
            let mut ret = 0;
            if state == crate::object::process::ProcessState::Zombie || state == crate::object::process::ProcessState::Crashed {
                if !status_ptr.is_null() {
                    if let Some(info) = *process.exit_info.lock() {
                        let info_bytes = unsafe { core::slice::from_raw_parts(&info as *const _ as *const u8, core::mem::size_of_val(&info)) };
                        crate::memory::user_copy::copy_to_user(info_bytes, status_ptr as *mut u8);
                    }
                }
                ret = state as u64; // Return the state
                
                // Add to history
                let snapshot = process.snapshot();
                let mut history = crate::object::history::HISTORIAN.lock();
                history.push_back(snapshot);
                if history.len() > 256 { history.pop_front(); }
                
                // Reap process!
                sched.remove_process(crate::object::process::ProcessId(target_pid));
                // Remove from registry so its name is freed
                let mut registry = crate::object::ipc::REGISTRY.lock();
                let mut to_remove = None;
                for (name, reg) in registry.iter() {
                    if reg.pid.0 == target_pid {
                        to_remove = Some(name.clone());
                        break;
                    }
                }
                if let Some(name) = to_remove {
                    registry.remove(&name);
                }
            } else {
                ret = 11; // EAGAIN / WouldBlock
            }
            frame.rax = ret;
        } else {
            frame.rax = 3; // ESRCH (No such process)
        }
    } else if frame.rax == Syscall::KillProcess as u64 {
        let target_pid = frame.rdi as u64;
        let sched = crate::task::scheduler::SCHEDULER.lock();
        if let Some(process) = sched.get_process(crate::object::process::ProcessId(target_pid)) {
            *process.state.lock() = crate::object::process::ProcessState::Crashed;
            *process.exit_info.lock() = Some(crate::object::process::ProcessExit {
                state: crate::object::process::ProcessState::Crashed,
                code: -9,
            });
            frame.rax = 0;
        } else {
            frame.rax = 3; // ESRCH
        }
    } else if frame.rax == Syscall::Snapshot as u64 {
        let target_pid = frame.rdi as u64;
        let snap_ptr = frame.rsi as *mut crate::object::process::ProcessSnapshot;
        let sched = crate::task::scheduler::SCHEDULER.lock();
        if let Some(process) = sched.get_process(crate::object::process::ProcessId(target_pid)) {
            let snapshot = process.snapshot();
            let snap_bytes = unsafe { core::slice::from_raw_parts(&snapshot as *const _ as *const u8, core::mem::size_of_val(&snapshot)) };
            if crate::memory::user_copy::copy_to_user(snap_bytes, snap_ptr as *mut u8) {
                frame.rax = 0;
            } else {
                frame.rax = 14; // EFAULT
            }
        } else {
            frame.rax = 3; // ESRCH
        }
    } else if frame.rax == Syscall::GetHistory as u64 {
        let buf_ptr = frame.rdi as *mut u8;
        let buf_size = frame.rsi as usize;
        
        let history = crate::object::history::HISTORIAN.lock();
        let mut count = 0;
        let snap_size = core::mem::size_of::<crate::object::history::Snapshot>();
        
        for snap in history.iter() {
            if count * snap_size + snap_size > buf_size {
                break;
            }
            let snap_bytes = unsafe { core::slice::from_raw_parts(snap as *const _ as *const u8, snap_size) };
            let dest_ptr = unsafe { buf_ptr.add(count * snap_size) };
            if !crate::memory::user_copy::copy_to_user(snap_bytes, dest_ptr) {
                break; // EFAULT
            }
            count += 1;
        }
    } else if frame.rax == Syscall::GetPciDevices as u64 {
        #[repr(C)]
        struct PciDeviceInfo {
            id: u64,
            bus: u8,
            device: u8,
            function: u8,
            vendor_id: u16,
            device_id: u16,
            class_code: u8,
            subclass: u8,
            prog_if: u8,
        }
        let buf_ptr = frame.rdi as *mut u8;
        let buf_size = frame.rsi as usize;
        
        let registry = crate::object::device::DEVICE_REGISTRY.lock();
        let mut count = 0;
        let pci_size = core::mem::size_of::<PciDeviceInfo>();
        
        for dev in registry.iter() {
            if count * pci_size + pci_size > buf_size { break; }
            let info = PciDeviceInfo {
                id: dev.id.0,
                bus: dev.hardware_info.bus,
                device: dev.hardware_info.device,
                function: dev.hardware_info.function,
                vendor_id: dev.hardware_info.vendor_id,
                device_id: dev.hardware_info.device_id,
                class_code: dev.hardware_info.class_code,
                subclass: dev.hardware_info.subclass,
                prog_if: dev.hardware_info.prog_if,
            };
            let bytes = unsafe { core::slice::from_raw_parts(&info as *const _ as *const u8, pci_size) };
            let dest_ptr = unsafe { buf_ptr.add(count * pci_size) };
            if !crate::memory::user_copy::copy_to_user(bytes, dest_ptr) { break; }
            count += 1;
        }
        frame.rax = count as u64;
    } else if frame.rax == Syscall::UpdateDeviceState as u64 {
        let dev_id = frame.rdi as u64;
        let state_val = frame.rsi as u64; // Value of DeviceState enum
        let name_ptr = frame.rdx as *const u8;
        
        let state = unsafe { core::mem::transmute::<u8, crate::object::device::DeviceState>(state_val as u8) };
        let registry = crate::object::device::DEVICE_REGISTRY.lock();
        
        if let Some(dev) = registry.iter().find(|d| d.id.0 == dev_id) {
            *dev.state.lock() = state;
            if !name_ptr.is_null() {
                let mut name_buf = [0u8; 16];
                // we should use a copy_from_user, but we don't have it. We'll use unsafe read.
                // It's dangerous without proper validation, but works for Phase 8.3 prototype.
                unsafe {
                    core::ptr::copy_nonoverlapping(name_ptr, name_buf.as_mut_ptr(), 16);
                }
                *dev.driver_name.lock() = name_buf;
            }
            
            // Log to HISTORIAN
            let snap = dev.snapshot();
            let mut history = crate::object::history::HISTORIAN.lock();
            history.push_back(snap);
            if history.len() > 256 { history.pop_front(); }
            
            frame.rax = 0;
        } else {
            frame.rax = 3; // ESRCH
        }
    } else if frame.rax == Syscall::OpenDevice as u64 {
        // Stub: Convert OpenDevice to an IPC message if needed
        frame.rax = 0; // Success mock
    } else if frame.rax == Syscall::ReadDevice as u64 {
        // Stub
        frame.rax = 0;
    } else if frame.rax == Syscall::WriteDevice as u64 {
        // Stub
        frame.rax = 0;
    } else if frame.rax == Syscall::Ioctl as u64 {
        // Stub
        frame.rax = 0;
    } else if frame.rax == Syscall::RegisterDriver as u64 {
        let name_ptr = frame.rdi as *const u8;
        let pid = frame.rsi as u64;
        let device_id = frame.rdx as u64;
        
        let mut name_buf = [0u8; 16];
        if !name_ptr.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(name_ptr, name_buf.as_mut_ptr(), 16);
            }
        }
        let name_str = alloc::string::String::from(core::str::from_utf8(&name_buf).unwrap_or("unknown").trim_matches('\0'));
        
        let mut registry = crate::object::driver::DRIVER_REGISTRY.lock();
        let driver_id = registry.len() as u64 + 1;
        let mut driver_obj = crate::object::driver::DriverObject::new(
            driver_id,
            name_str,
            device_id
        );
        
        // Setup service handle
        driver_obj.service_handle = Some(pid);
        let driver = alloc::sync::Arc::new(driver_obj);
        registry.push(driver.clone());
        
        // Push snapshot
        let snap = driver.snapshot();
        let mut history = crate::object::history::HISTORIAN.lock();
        history.push_back(snap);
        if history.len() > 256 { history.pop_front(); }
        
        frame.rax = driver_id;
    } else if frame.rax == Syscall::UpdateDriverState as u64 {
        let driver_id = frame.rdi as u64;
        let state_val = frame.rsi as u8;
        
        let state = unsafe { core::mem::transmute::<u8, crate::object::driver::DriverState>(state_val) };
        let registry = crate::object::driver::DRIVER_REGISTRY.lock();
        
        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        let current_pid = if !cpu_local.current_process.is_null() {
            unsafe { (*cpu_local.current_process).id.0 }
        } else { 0 };
        
        let target_driver = if driver_id == 0 {
            registry.iter().find(|d| d.service_handle == Some(current_pid))
        } else {
            registry.iter().find(|d| d.id == driver_id)
        };
        
        if let Some(drv) = target_driver {
            *drv.state.lock() = state;
            
            // Push snapshot
            let snap = drv.snapshot();
            let mut history = crate::object::history::HISTORIAN.lock();
            history.push_back(snap);
            if history.len() > 256 { history.pop_front(); }
            
            frame.rax = 0;
        } else {
            frame.rax = 3; // ESRCH
        }
    } else if frame.rax == Syscall::Open as u64 {
        let path_ptr = frame.rdi as *const u8;
        let path_len = frame.rsi as usize;
        let mode = frame.rdx as u32;

        let mut path_buf = alloc::vec::Vec::with_capacity(path_len);
        unsafe {
            path_buf.set_len(path_len);
            core::ptr::copy_nonoverlapping(path_ptr, path_buf.as_mut_ptr(), path_len);
        }
        let path_str = core::str::from_utf8(&path_buf).unwrap_or("/");

        let mount_mgr = crate::vfs::mount::MOUNT_MANAGER.lock();
        match mount_mgr.resolve_path(path_str) {
            Ok(vnode) => {
                let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
                if cpu_local.current_process.is_null() {
                    frame.rax = 0;
                    return;
                }
                let process = unsafe { &*cpu_local.current_process };
                
                // Security Check
                if let Err(_) = crate::security::policy::AccessManager::check_access(&process.token, &vnode.security.lock(), mode) {
                    frame.rax = 0; // Access Denied (return 0 handle)
                    return;
                }

                let file_obj = alloc::sync::Arc::new(crate::vfs::FileObject::new(vnode, mode));
                let rights = crate::object::handle::HandleRights::from_bits_truncate(mode);
                let handle = process.handle_table.add(file_obj, rights);
                frame.rax = handle.0;
            }
            Err(_) => {
                frame.rax = 0; // Error / Not found
            }
        }
    } else if frame.rax == Syscall::Read as u64 {
        let handle_id = frame.rdi as u64;
        let buf_ptr = frame.rsi as *mut u8;
        let buf_len = frame.rdx as usize;

        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        if cpu_local.current_process.is_null() { frame.rax = 0; return; }
        let process = unsafe { &*cpu_local.current_process };

        if let Some(entry) = process.handle_table.get(crate::object::handle::Handle(handle_id)) {
            if let Some(file_obj) = entry.object.as_any().downcast_ref::<crate::vfs::FileObject>() {
                let mut buf = alloc::vec::Vec::with_capacity(buf_len);
                unsafe { buf.set_len(buf_len); }
                
                let mut cursor = file_obj.cursor.lock();
                match file_obj.vnode.fs.read(&file_obj.vnode, *cursor, &mut buf) {
                    Ok(bytes_read) => {
                        unsafe {
                            core::ptr::copy_nonoverlapping(buf.as_ptr(), buf_ptr, bytes_read);
                        }
                        *cursor += bytes_read as u64;
                        frame.rax = bytes_read as u64;
                    }
                    Err(_) => frame.rax = 0,
                }
            } else {
                frame.rax = 0;
            }
        } else {
            frame.rax = 0;
        }
    } else if frame.rax == Syscall::Write as u64 {
        let handle_id = frame.rdi as u64;
        let buf_ptr = frame.rsi as *const u8;
        let buf_len = frame.rdx as usize;

        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        if cpu_local.current_process.is_null() { frame.rax = 0; return; }
        let process = unsafe { &*cpu_local.current_process };

        if let Some(entry) = process.handle_table.get(crate::object::handle::Handle(handle_id)) {
            if let Some(file_obj) = entry.object.as_any().downcast_ref::<crate::vfs::FileObject>() {
                let mut buf = alloc::vec::Vec::with_capacity(buf_len);
                unsafe {
                    buf.set_len(buf_len);
                    core::ptr::copy_nonoverlapping(buf_ptr, buf.as_mut_ptr(), buf_len);
                }
                
                let mut cursor = file_obj.cursor.lock();
                match file_obj.vnode.fs.write(&file_obj.vnode, *cursor, &buf) {
                    Ok(bytes_written) => {
                        *cursor += bytes_written as u64;
                        frame.rax = bytes_written as u64;
                    }
                    Err(_) => frame.rax = 0,
                }
            } else {
                frame.rax = 0;
            }
        } else {
            frame.rax = 0;
        }
    } else if frame.rax == Syscall::Close as u64 {
        let handle_id = frame.rdi as u64;
        let cpu_local = unsafe { super::cpu_local::get_cpu_local_mut() };
        if cpu_local.current_process.is_null() { frame.rax = 0; return; }
        let process = unsafe { &*cpu_local.current_process };
        
        if process.handle_table.remove(crate::object::handle::Handle(handle_id)).is_some() {
            frame.rax = 0;
        } else {
            frame.rax = 1;
        }
    } else if frame.rax == Syscall::Mkdir as u64 {
        let path_ptr = frame.rdi as *const u8;
        let path_len = frame.rsi as usize;
        let _mode = frame.rdx as u32;

        let mut path_buf = alloc::vec::Vec::with_capacity(path_len);
        unsafe {
            path_buf.set_len(path_len);
            core::ptr::copy_nonoverlapping(path_ptr, path_buf.as_mut_ptr(), path_len);
        }
        let path_str = core::str::from_utf8(&path_buf).unwrap_or("/");

        let mount_mgr = crate::vfs::mount::MOUNT_MANAGER.lock();
        
        let mut parent_path = path_str;
        let mut dir_name = "";
        if let Some(idx) = path_str.rfind('/') {
            parent_path = if idx == 0 { "/" } else { &path_str[..idx] };
            dir_name = &path_str[idx+1..];
        }

        if dir_name.is_empty() {
            frame.rax = 1; // Error
            return;
        }

        match mount_mgr.resolve_path(parent_path) {
            Ok(parent_vnode) => {
                match parent_vnode.fs.mkdir(&parent_vnode, dir_name, crate::security::SecurityDescriptor::system_default()) {
                    Ok(_) => frame.rax = 0,
                    Err(_) => frame.rax = 1,
                }
            }
            Err(_) => frame.rax = 1,
        }
    } else {
        crate::println!("Unknown syscall!");
        frame.rax = !0;
    }
}
