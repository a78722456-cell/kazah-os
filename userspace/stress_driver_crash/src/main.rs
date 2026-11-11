#![no_std]
#![no_main]

use libkazah::syscalls::{exit, find, send, yield_cpu};
use libkazah::driver::IpcMessageId;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    
    for _ in 0..1000 {
        // Find driver
        let mut handle = 2; // ENOENT
        for _ in 0..5000 {
            handle = find("ahci_driver");
            if handle != 2 { break; }
            yield_cpu();
        }
        
        if handle != 2 {
            // Send crash message
            let msg = [IpcMessageId::CrashDriver as u8; 1];
            send(handle, &msg);
            
            // Wait for it to die
            for _ in 0..50 { yield_cpu(); }
        }
    }
    
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
