#![no_std]
#![no_main]
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let handle = libkazah::syscalls::register("stress_ipc");
    
    // Test 100_000 messages send and recv in sequence (to self)
    for _ in 0..100_000 {
        let msg = b"ping";
        while libkazah::syscalls::send(handle, msg) != 0 {
            libkazah::syscalls::yield_cpu();
        }
        
        let mut buf = [0u8; 16];
        while libkazah::syscalls::recv(handle, &mut buf) != 0 {
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
