#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libkazah::process::{ProcessExit, ProcessState};

fn wait_for_process(pid: u64) {
    let mut exit_info = ProcessExit { state: ProcessState::Running, code: 0 };
    loop {
        let status = libkazah::syscalls::wait(pid, &mut exit_info);
        if status == ProcessState::Zombie as u64 || status == ProcessState::Crashed as u64 {
            break;
        }
        libkazah::syscalls::yield_cpu();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let _ret = libkazah::syscalls::register("service_manager");
    
    // 0. Spawning DeviceManager & DriverManager First
    let _dm_pid = libkazah::syscalls::spawn("device_manager");
    libkazah::syscalls::yield_cpu(); // Let it initialize

    let _drm_pid = libkazah::syscalls::spawn("driver_manager");
    libkazah::syscalls::yield_cpu();

    let _rd_pid = libkazah::syscalls::spawn("ramdisk");
    libkazah::syscalls::yield_cpu();
    
    // 1. Stress Spawn
    let pid = libkazah::syscalls::spawn("stress_spawn");
    wait_for_process(pid);

    // 2. Stress Crash
    let pid = libkazah::syscalls::spawn("stress_crash");
    wait_for_process(pid);
    
    // 3. Stress IPC
    let pid = libkazah::syscalls::spawn("stress_ipc");
    wait_for_process(pid);
    
    // 4. Stress Kill
    let pid = libkazah::syscalls::spawn("stress_kill");
    wait_for_process(pid);
    
    // 5. Phase 8.3 Stress Tests
    let pid = libkazah::syscalls::spawn("stress_pci");
    wait_for_process(pid);

    let pid = libkazah::syscalls::spawn("stress_device_snap");
    wait_for_process(pid);

    let pid = libkazah::syscalls::spawn("stress_registry");
    wait_for_process(pid);

    let pid = libkazah::syscalls::spawn("stress_churn");
    wait_for_process(pid);
    
    // Phase 8.4 Stress Tests
    let pid = libkazah::syscalls::spawn("stress_driver_crash");
    wait_for_process(pid);

    let pid = libkazah::syscalls::spawn("stress_driver_rebind");
    wait_for_process(pid);

    // Phase 8.5: Storage layer
    let _ = libkazah::syscalls::spawn("ahci");
    // Phase 8.6: Volume Manager
    let _ = libkazah::syscalls::spawn("volume_manager");
    
    // Allow drivers to initialize
    for _ in 0..100 {
        libkazah::syscalls::yield_cpu();
    }
    
    // Spawn Stress Tests
    let pid = libkazah::syscalls::spawn("stress_security");
    wait_for_process(pid);
    let pid = libkazah::syscalls::spawn("stress_vfs");
    wait_for_process(pid);
    let pid = libkazah::syscalls::spawn("stress_volume_manager");
    wait_for_process(pid);
    let pid = libkazah::syscalls::spawn("stress_storage_flood");
    wait_for_process(pid);
    let pid = libkazah::syscalls::spawn("stress_storage_reset");
    wait_for_process(pid);
    let pid = libkazah::syscalls::spawn("stress_storage_concurrent");
    wait_for_process(pid);
    
    // 6. Stress Mem (Memory Exhaustion) - run last
    let pid = libkazah::syscalls::spawn("stress_mem");
    wait_for_process(pid);
    
    loop {
        libkazah::syscalls::yield_cpu();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    libkazah::syscalls::exit(-1);
    loop {}
}
