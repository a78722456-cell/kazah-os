#![no_std]
#![no_main]

use libkazah::syscalls::{exit, find, send, yield_cpu};
use libkazah::storage::{VolumeOpcode, VolumeRequestHeader};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut handle = 2; // ENOENT
    for _ in 0..100 {
        handle = find("volume_manager");
        if handle != 2 {
            break;
        }
        yield_cpu();
    }
    
    if handle == 2 {
        exit(-1);
    }
    
    // Simulate Event Bus subscription and rapid Volume Creation/Deletion requests
    libkazah::event_bus::subscribe_events();
    
    for i in 0..1000 {
        let req = VolumeRequestHeader {
            opcode: VolumeOpcode::Create as u8,
            request_id: i,
            partition_id: 1,
            lba: i * 10,
            count: 10,
        };
        let buf = unsafe { core::slice::from_raw_parts(&req as *const _ as *const u8, core::mem::size_of::<VolumeRequestHeader>()) };
        send(handle, buf);
        yield_cpu();
    }
    
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
