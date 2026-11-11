#![no_std]
#![no_main]

use libkazah::syscalls::{exit, register, recv, send, update_driver_state, yield_cpu};
use libkazah::driver::{IpcMessageId, DriverState};
use libkazah::storage::{BlockDevice, StorageOpcode, StorageRequestHeader, StorageResponseHeader, StorageIdentifyInfo};

struct RamDisk {
    storage: [u8; 65536], // 128 blocks of 512 bytes
}

static mut RAM_DISK: RamDisk = RamDisk { storage: [0u8; 65536] };

impl BlockDevice for RamDisk {
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> i32 {
        let start = (lba as usize) * 512;
        let len = (count as usize) * 512;
        if start + len > self.storage.len() || buffer.len() < len {
            return -1;
        }
        buffer[..len].copy_from_slice(&self.storage[start..start + len]);
        0
    }

    fn write_blocks(&self, lba: u64, count: u32, buffer: &[u8]) -> i32 {
        let start = (lba as usize) * 512;
        let len = (count as usize) * 512;
        unsafe {
            let disk = core::ptr::addr_of_mut!(RAM_DISK);
            if start + len > (*disk).storage.len() || buffer.len() < len {
                return -1;
            }
            (&mut (*disk).storage)[start..start + len].copy_from_slice(&buffer[..len]);
        }
        0
    }

    fn flush(&self) -> i32 {
        0
    }

    fn block_size(&self) -> u32 {
        512
    }

    fn block_count(&self) -> u64 {
        128
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let chan = register("ramdisk_driver");
    if chan == 17 { // EEXIST
        exit(1);
    }

    let mut buf = [0u8; 4096];
    loop {
        let ret = recv(chan, &mut buf);
        if ret > 0 && ret < 4096 {
            if buf[0] == IpcMessageId::BindDevice as u8 {
                update_driver_state(1, DriverState::Binding);
                yield_cpu();
                update_driver_state(1, DriverState::Running);
            } else if buf[0] == IpcMessageId::CrashDriver as u8 {
                exit(-9);
            } else if buf[0] == IpcMessageId::UnbindDevice as u8 {
                update_driver_state(1, DriverState::Unloaded);
            } else if buf[0] == StorageOpcode::Identify as u8 {
                let info = StorageIdentifyInfo {
                    block_size: 512,
                    block_count: 128,
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&info as *const _ as *const u8, core::mem::size_of::<StorageIdentifyInfo>()) };
                send(chan, resp_buf);
            } else if buf[0] == StorageOpcode::Read as u8 {
                let req = unsafe { core::ptr::read(buf.as_ptr() as *const StorageRequestHeader) };
                let mut data_buf = [0u8; 512];
                let disk = core::ptr::addr_of!(RAM_DISK);
                let status = unsafe { (*disk).read_blocks(req.lba, req.count, &mut data_buf) };
                let resp = StorageResponseHeader {
                    request_id: req.request_id,
                    status,
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&resp as *const _ as *const u8, core::mem::size_of::<StorageResponseHeader>()) };
                send(chan, resp_buf);
            } else if buf[0] == StorageOpcode::Write as u8 {
                let req = unsafe { core::ptr::read(buf.as_ptr() as *const StorageRequestHeader) };
                let end_offset = core::mem::size_of::<StorageRequestHeader>() + (req.count as usize * 512);
                let status = if end_offset <= buf.len() {
                    let data_slice = &buf[core::mem::size_of::<StorageRequestHeader>()..end_offset];
                    let disk = core::ptr::addr_of!(RAM_DISK);
                    unsafe { (*disk).write_blocks(req.lba, req.count, data_slice) }
                } else {
                    -1
                };
                let resp = StorageResponseHeader {
                    request_id: req.request_id,
                    status,
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&resp as *const _ as *const u8, core::mem::size_of::<StorageResponseHeader>()) };
                send(chan, resp_buf);
            } else if buf[0] == libkazah::storage::VolumeOpcode::Create as u8 || buf[0] == libkazah::storage::VolumeOpcode::Delete as u8 {
                let req = unsafe { core::ptr::read(buf.as_ptr() as *const libkazah::storage::VolumeRequestHeader) };
                let resp = libkazah::storage::VolumeResponseHeader {
                    request_id: req.request_id,
                    status: 0, // Mock success
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&resp as *const _ as *const u8, core::mem::size_of::<libkazah::storage::VolumeResponseHeader>()) };
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
