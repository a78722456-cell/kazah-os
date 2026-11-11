#![no_std]

pub mod process {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessState {
        Starting,
        Running,
        Waiting,
        Stopped,
        Crashed,
        Zombie,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ProcessExit {
        pub state: ProcessState,
        pub code: i32,
    }

    #[derive(Debug, Clone)]
    pub struct ProcessSnapshot {
        pub header: crate::history::SnapshotHeader,
        pub state: ProcessState,
        pub uptime_ticks: u64,
        pub restart_count: u32,
        pub open_handles_count: usize,
        pub thread_count: usize,
        pub cr3_address: u64,
        pub image_name: [u8; 64],
    }
}

pub mod history;
pub mod device;
pub mod event_bus;

pub mod driver;
pub mod storage;

pub mod syscalls {
    pub const SYS_EXIT: u64 = 0;
    pub const SYS_YIELD: u64 = 1;
    pub const SYS_SEND: u64 = 2;
    pub const SYS_RECV: u64 = 3;
    pub const SYS_REGISTER: u64 = 4;
    pub const SYS_FIND: u64 = 5;
    pub const SYS_SPAWN: u64 = 6;
    pub const SYS_KILL: u64 = 7;
    pub const SYS_WAIT: u64 = 8;
    pub const SYS_SNAPSHOT: u64 = 9;
    pub const SYS_GETHISTORY: u64 = 10;
    pub const SYS_GETPCIDEVICES: u64 = 11;
    pub const SYS_UPDATEDEVICESTATE: u64 = 12;
    pub const SYS_OPENDEVICE: u64 = 13;
    pub const SYS_READDEVICE: u64 = 14;
    pub const SYS_WRITEDEVICE: u64 = 15;
    pub const SYS_IOCTL: u64 = 16;
    pub const SYS_REGISTERDRIVER: u64 = 17;
    pub const SYS_UPDATEDRIVERSTATE: u64 = 18;
    pub const SYS_OPEN: u64 = 19;
    pub const SYS_READ: u64 = 20;
    pub const SYS_WRITE: u64 = 21;
    pub const SYS_CLOSE: u64 = 22;
    pub const SYS_MKDIR: u64 = 23;
    pub const SYS_MOUNT: u64 = 24;
    pub fn exit(code: i32) -> ! {
        unsafe { core::arch::asm!("syscall", in("rax") SYS_EXIT, in("rdi") code, out("rcx") _, out("r11") _) }
        loop {}
    }
    pub fn yield_cpu() {
        unsafe { core::arch::asm!("syscall", in("rax") SYS_YIELD, out("rcx") _, out("r11") _) }
    }
    pub fn send(handle: u64, data: &[u8]) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_SEND,
                in("rdi") handle,
                in("rsi") data.as_ptr(),
                in("rdx") data.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn recv(handle: u64, buf: &mut [u8]) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_RECV,
                in("rdi") handle,
                in("rsi") buf.as_mut_ptr(),
                in("rdx") buf.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn register(name: &str) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_REGISTER,
                in("rdi") name.as_ptr(),
                in("rsi") name.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn find(name: &str) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_FIND,
                in("rdi") name.as_ptr(),
                in("rsi") name.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn spawn(name: &str) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_SPAWN,
                in("rdi") name.as_ptr(),
                in("rsi") name.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn kill(pid: u64) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_KILL,
                in("rdi") pid,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn wait(pid: u64, exit_info: &mut super::process::ProcessExit) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_WAIT,
                in("rdi") pid,
                in("rsi") exit_info as *mut _,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn snapshot(pid: u64, snap: &mut super::process::ProcessSnapshot) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_SNAPSHOT,
                in("rdi") pid,
                in("rsi") snap as *mut _,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn get_history(buf: &mut [super::history::SnapshotRecord]) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_GETHISTORY,
                in("rdi") buf.as_mut_ptr(),
                in("rsi") buf.len() * core::mem::size_of::<super::history::SnapshotRecord>(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
    pub fn get_pci_devices(buf: &mut [super::device::PciDeviceInfo]) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_GETPCIDEVICES,
                in("rdi") buf.as_mut_ptr(),
                in("rsi") buf.len() * core::mem::size_of::<super::device::PciDeviceInfo>(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn update_device_state(dev_id: u64, state: super::device::DeviceState, name: Option<&[u8; 16]>) -> u64 {
        let mut ret: u64;
        let name_ptr = if let Some(n) = name { n.as_ptr() } else { core::ptr::null() };
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_UPDATEDEVICESTATE,
                in("rdi") dev_id,
                in("rsi") state as u64,
                in("rdx") name_ptr,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn register_driver(name: Option<&[u8; 16]>, pid: u64, device_id: u64) -> u64 {
        let mut ret: u64;
        let name_ptr = if let Some(n) = name { n.as_ptr() } else { core::ptr::null() };
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_REGISTERDRIVER,
                in("rdi") name_ptr,
                in("rsi") pid,
                in("rdx") device_id,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn update_driver_state(driver_id: u64, state: super::driver::DriverState) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_UPDATEDRIVERSTATE,
                in("rdi") driver_id,
                in("rsi") state as u64,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn open(path: &str, mode: u32) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_OPEN,
                in("rdi") path.as_ptr(),
                in("rsi") path.len(),
                in("rdx") mode,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn read(handle: u64, buf: &mut [u8]) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_READ,
                in("rdi") handle,
                in("rsi") buf.as_mut_ptr(),
                in("rdx") buf.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn write(handle: u64, buf: &[u8]) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_WRITE,
                in("rdi") handle,
                in("rsi") buf.as_ptr(),
                in("rdx") buf.len(),
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn close(handle: u64) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_CLOSE,
                in("rdi") handle,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }

    pub fn mkdir(path: &str, mode: u32) -> u64 {
        let mut ret: u64;
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") SYS_MKDIR,
                in("rdi") path.as_ptr(),
                in("rsi") path.len(),
                in("rdx") mode,
                out("rcx") _, out("r11") _, lateout("rax") ret,
            );
        }
        ret
    }
}
