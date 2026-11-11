#![no_std]
#![no_main]

use libkazah::syscalls::{exit, register, recv, send, yield_cpu};
use libkazah::driver::IpcMessageId;
use libkazah::storage::{VolumeOpcode, VolumeRequestHeader, VolumeResponseHeader};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Register as Volume Manager Service
    let chan = register("volume_manager");
    if chan == 17 { // EEXIST
        exit(1);
    }
    
    // Subscribe to Storage Event Bus
    // In a real implementation, we would listen for StorageDeviceConnected
    libkazah::event_bus::subscribe_events();

    let mut buf = [0u8; 4096];
    loop {
        let ret = recv(chan, &mut buf);
        if ret > 0 && ret < 4096 {
            if buf[0] == VolumeOpcode::Create as u8 {
                let req = unsafe { core::ptr::read(buf.as_ptr() as *const VolumeRequestHeader) };
                
                // Stub: dynamically "create" volume by sending success
                let resp = VolumeResponseHeader {
                    request_id: req.request_id,
                    status: 0,
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&resp as *const _ as *const u8, core::mem::size_of::<VolumeResponseHeader>()) };
                send(chan, resp_buf);
            }
        }
        yield_cpu();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
