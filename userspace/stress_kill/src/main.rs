#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    libkazah::syscalls::register("stress_kill");
    
    for _ in 0..500 {
        let pid = libkazah::syscalls::spawn("test_service");
        libkazah::syscalls::kill(pid);
        
        let mut exit_info = libkazah::process::ProcessExit { state: libkazah::process::ProcessState::Running, code: 0 };
        loop {
            let status = libkazah::syscalls::wait(pid, &mut exit_info);
            if status == libkazah::process::ProcessState::Zombie as u64 || status == libkazah::process::ProcessState::Crashed as u64 {
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
