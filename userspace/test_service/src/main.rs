#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    libkazah::syscalls::register("test_service");
    
    // Simulate some work
    for _ in 0..10_000_000 {
        libkazah::syscalls::yield_cpu();
    }
    
    // Exit successfully
    libkazah::syscalls::exit(0);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    libkazah::syscalls::exit(-1);
    loop {}
}
