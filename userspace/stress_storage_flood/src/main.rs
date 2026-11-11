#![no_std]
#![no_main]

use libkazah::syscalls::{exit, find, send, yield_cpu};
use libkazah::storage::{StorageOpcode, StorageRequestHeader};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut handle = 2; // ENOENT
    for _ in 0..5000 {
        handle = find("ramdisk_driver");
        if handle != 2 { break; }
        yield_cpu();
    }

    if handle != 2 {
        for i in 0..1000 {
            let req = StorageRequestHeader {
                opcode: StorageOpcode::Read as u8,
                request_id: i,
                lba: (i % 128) as u64,
                count: 1,
            };
            let req_buf = unsafe { core::slice::from_raw_parts(&req as *const _ as *const u8, core::mem::size_of::<StorageRequestHeader>()) };
            send(handle, req_buf);
            yield_cpu();
        }
    }

    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
