#![no_std]
#![no_main]

use libkazah::syscalls::{exit, find, send, yield_cpu};
use libkazah::driver::{BindDeviceMsg, IpcMessageId};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    
    // Find driver
    let mut handle = 2; // ENOENT
    for _ in 0..5000 {
        handle = find("ahci_driver");
        if handle != 2 { break; }
        yield_cpu();
    }
    
    if handle != 2 {
        for _ in 0..10 {
            // Unbind
            let msg1 = [IpcMessageId::UnbindDevice as u8; 1];
            send(handle, &msg1);
            for _ in 0..10 { yield_cpu(); }
            
            // Rebind
            let msg2 = BindDeviceMsg {
                msg_id: IpcMessageId::BindDevice as u8,
                device_id: 1, // Mock device ID
            };
            let bytes = unsafe { core::slice::from_raw_parts(&msg2 as *const _ as *const u8, core::mem::size_of::<BindDeviceMsg>()) };
            send(handle, bytes);
            for _ in 0..10 { yield_cpu(); }
        }
    }
    
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
