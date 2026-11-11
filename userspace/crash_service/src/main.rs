#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    libkazah::syscalls::register("crash_service");
    
    // Simulate some work
    for _ in 0..2_000_000 {
        libkazah::syscalls::yield_cpu();
    }
    
    // Intentional page fault (crash)
    unsafe {
        let ptr = 0x0 as *mut u64;
        *ptr = 1; // Page Fault!
    }
    
    libkazah::syscalls::exit(0);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    libkazah::syscalls::exit(-1);
    loop {}
}
