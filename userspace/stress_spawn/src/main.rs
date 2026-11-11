#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libkazah::process::{ProcessExit, ProcessState};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    libkazah::syscalls::register("stress_spawn");
    
    for _ in 0..1000 {
        let pid = libkazah::syscalls::spawn("test_service");
        let mut exit_info = ProcessExit { state: ProcessState::Running, code: 0 };
        loop {
            let status = libkazah::syscalls::wait(pid, &mut exit_info);
            if status == ProcessState::Zombie as u64 || status == ProcessState::Crashed as u64 {
                break;
            }
            libkazah::syscalls::yield_cpu();
        }
    }
    
    libkazah::syscalls::exit(0);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    libkazah::syscalls::exit(-1);
    loop {}
}
