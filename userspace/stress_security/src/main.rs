#![no_std]
#![no_main]

use libkazah::syscalls::exit;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Basic test to see if the process starts and has its SecurityToken evaluated.
    
    // For now, we just exit with 0.
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
