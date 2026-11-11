#![no_std]
#![no_main]

use libkazah::syscalls::{exit, register, recv, send, update_driver_state, yield_cpu};
use libkazah::driver::{IpcMessageId, DriverState};
use libkazah::storage::{BlockDevice, StorageOpcode, StorageRequestHeader, StorageResponseHeader, StorageIdentifyInfo};

struct AhciDevice {
    sectors: [u8; 8192], // Simulate 16 blocks of 512 bytes for DMA buffer testing
}

static mut AHCI_DEV: AhciDevice = AhciDevice { sectors: [0u8; 8192] };

impl BlockDevice for AhciDevice {
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> i32 {
        let start = (lba as usize) * 512;
        let len = (count as usize) * 512;
        if start + len > self.sectors.len() || buffer.len() < len {
            return -1;
        }
        buffer[..len].copy_from_slice(&self.sectors[start..start + len]);
        0
    }

    fn write_blocks(&self, lba: u64, count: u32, buffer: &[u8]) -> i32 {
        let start = (lba as usize) * 512;
        let len = (count as usize) * 512;
        unsafe {
            let dev = core::ptr::addr_of_mut!(AHCI_DEV);
            if start + len > (*dev).sectors.len() || buffer.len() < len {
                return -1;
            }
            (&mut (*dev).sectors)[start..start + len].copy_from_slice(&buffer[..len]);
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
        1024 // Reported total capacity
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let chan = register("ahci_driver");
    if chan == 17 { // EEXIST
        exit(1);
    }
    
    let mut buf = [0u8; 4096];
    loop {
        let ret = recv(chan, &mut buf);
        if ret > 0 && ret < 4096 {
            if buf[0] == IpcMessageId::BindDevice as u8 {
                update_driver_state(0, DriverState::Binding);
                yield_cpu();
                update_driver_state(0, DriverState::Running);
            } else if buf[0] == IpcMessageId::CrashDriver as u8 {
                exit(-9);
            } else if buf[0] == IpcMessageId::UnbindDevice as u8 {
                update_driver_state(0, DriverState::Unloaded);
            } else if buf[0] == StorageOpcode::Identify as u8 {
                let info = StorageIdentifyInfo {
                    block_size: 512,
                    block_count: 1024,
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&info as *const _ as *const u8, core::mem::size_of::<StorageIdentifyInfo>()) };
                send(chan, resp_buf);
            } else if buf[0] == StorageOpcode::Read as u8 {
                let req = unsafe { core::ptr::read(buf.as_ptr() as *const StorageRequestHeader) };
                let mut data_buf = [0u8; 512];
                let dev = core::ptr::addr_of!(AHCI_DEV);
                let status = unsafe { (*dev).read_blocks(req.lba, req.count, &mut data_buf) };
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
                    let dev = core::ptr::addr_of!(AHCI_DEV);
                    unsafe { (*dev).write_blocks(req.lba, req.count, data_slice) }
                } else {
                    -1
                };
                let resp = StorageResponseHeader {
                    request_id: req.request_id,
                    status,
                };
                let resp_buf = unsafe { core::slice::from_raw_parts(&resp as *const _ as *const u8, core::mem::size_of::<StorageResponseHeader>()) };
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
