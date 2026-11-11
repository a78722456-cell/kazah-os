#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    libkazah::syscalls::register("stress_mem");
    
    // Spawn until ENOMEM (12)
    let mut spawned = 0;
    loop {
        let ret = libkazah::syscalls::spawn("test_service");
        if ret == 12 { // ENOMEM
            // Successfully exhausted memory without panic!
            break;
        } else if ret == 2 || ret == 8 {
            // Some other error
            libkazah::syscalls::exit(-2);
        }
        spawned += 1;
        
        // Yield occasionally
        if spawned % 50 == 0 {
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
