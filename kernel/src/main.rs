#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;
use limine::request::{HhdmRequest, MemmapRequest};
use limine::BaseRevision;

// Set the base revision to 1, this is recommended for new Limine projects
#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".requests"]
static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub mod serial;
pub mod interrupts;
pub mod memory;
pub mod task;
pub mod arch;
pub mod object;
pub mod security;
pub mod vfs;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Booting KAZAH OS...");

    // 1. Detect CPU Features & Init FPU (XSAVE/SSE)
    // 3. Setup CPU Features
    let features = arch::x86_64::cpu_features::CpuFeatures::detect();
    arch::x86_64::cpu_features::init_fpu(&features);
    arch::x86_64::cpu_features::init_security(&features);
    println!("CPU Features and Security (NX, SMEP, SMAP) initialized.");

    // 4. Initialize CpuLocal and Syscall
    arch::x86_64::cpu_local::init();
    let selectors = interrupts::gdt::get_selectors();
    arch::x86_64::syscall::init(
        selectors.code_selector,
        selectors.data_selector,
        selectors.user_code_selector,
        selectors.user_data_selector,
    );
    println!("CpuLocal and Syscall initialized.");
    println!("CPU Features detected: XSAVE={}, AVX={}", features.xsave, features.avx);

    // 2. Serial port is initialized lazily via macro
    println!("KAZAH OS kernel started.");

    // 1. Get memory map
    let memmap = MEMMAP_REQUEST.response().expect("No memory map provided by bootloader");
    
    // 2. Get Higher Half Direct Map offset
    let hhdm = HHDM_REQUEST.response().expect("No HHDM provided by bootloader");
    let hhdm_offset = hhdm.offset;

    // 3. Initialize Memory
    memory::init(memmap, hhdm_offset);
    println!("Memory initialized.");

    // 4. Initialize Interrupts (IDT, GDT, PIC)
    interrupts::init();
    println!("Interrupts initialized.");

    // 4.5 PCI Enumeration
    arch::x86_64::pci::enumerate_pci();
    println!("PCI Bus Enumerated.");

    // 4.6 VFS Initialization
    let tmpfs = vfs::tmpfs::TmpFs::new();
    let devfs = vfs::devfs::DevFs::new();
    vfs::mount::MOUNT_MANAGER.lock().mount("/", tmpfs);
    vfs::mount::MOUNT_MANAGER.lock().mount("/dev", devfs);
    println!("VFS Initialized (mounted / and /dev).");

    use alloc::sync::Arc;
    use object::thread::ThreadObject;
    use object::process::ProcessObject;
    
    // 5. Load Initrd and Service Manager
    let initrd_data = include_bytes!("../../build/initrd.tar");
    unsafe { crate::memory::tar::INITRD_DATA = Some(initrd_data); }
    let archive = crate::memory::tar::TarArchive::new(initrd_data);

    if let Some(elf_data) = archive.find("service_manager") {
        println!("Found service_manager in initrd! Loading ELF...");
        if let Some(mut process_sm) = ProcessObject::new(true, "service_manager") {
            if let Some(image) = crate::memory::elf::load_elf(elf_data, &mut process_sm) {
                println!("Service Manager loaded. Entry: {:#X}", image.entry_point);
                let process_arc = Arc::new(process_sm);
            
                task::scheduler::SCHEDULER.lock().add_process(process_arc.clone());
                task::scheduler::SCHEDULER.lock().set_idle_task(Arc::new(ThreadObject::new(task::IdleThread::run as *const () as usize as u64, alloc::sync::Arc::downgrade(&process_arc))));
                
                let thread = Arc::new(ThreadObject::new_user(image.entry_point, alloc::sync::Arc::downgrade(&process_arc)));
                task::scheduler::add_task(thread);
                
                println!("Service Manager Scheduled. Waiting for preempt...");
            } else {
                println!("Failed to load service_manager ELF!");
            }
        } else {
            println!("Failed to create ProcessObject (ENOMEM)!");
        }
    } else {
        println!("service_manager not found in initrd!");
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
