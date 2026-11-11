#![no_std]
#![no_main]

use libkazah::syscalls::{exit, register, get_pci_devices, spawn, wait, yield_cpu, register_driver, update_driver_state, find, send};
use libkazah::device::PciDeviceInfo;
use libkazah::process::{ProcessExit, ProcessState};
use libkazah::driver::{BindDeviceMsg, IpcMessageId, DriverState};

#[derive(Copy, Clone)]
struct ActiveDriver {
    name: &'static str,
    pid: u64,
    device_id: u64,
    driver_id: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let _chan = register("driver_manager");
    
    // 1. Get PCI Devices
    let mut devices = [PciDeviceInfo {
        id: 0, bus: 0, device: 0, function: 0, vendor_id: 0, device_id: 0, class_code: 0, subclass: 0, prog_if: 0,
    }; 32];
    
    let count = get_pci_devices(&mut devices);
    let mut active_drivers = [ActiveDriver { name: "", pid: 0, device_id: 0, driver_id: 0 }; 32];
    let mut driver_count = 0;
    
    // 2. Hardcoded manifest matching
    for i in 0..count as usize {
        let dev = &devices[i];
        
        // 0x01 = Mass Storage, 0x06 = SATA
        if dev.class_code == 0x01 && dev.subclass == 0x06 {
            let driver_name = b"ahci\0\0\0\0\0\0\0\0\0\0\0\0";
            let pid = spawn("ahci");
            
            // Register in Kernel DRIVER_REGISTRY
            let driver_id = register_driver(Some(driver_name), pid, dev.id);
            
            active_drivers[driver_count] = ActiveDriver {
                name: "ahci",
                pid,
                device_id: dev.id,
                driver_id,
            };
            driver_count += 1;
            
            bind_driver("ahci_driver", dev.id);
        }
    }
    
    // 3. Monitor and Restart
    let mut exit_info = ProcessExit { state: ProcessState::Running, code: 0 };
    loop {
        for i in 0..driver_count {
            let drv = &mut active_drivers[i];
            let status = wait(drv.pid, &mut exit_info);
            
            if status == ProcessState::Zombie as u64 || status == ProcessState::Crashed as u64 {
                // Driver died!
                update_driver_state(drv.driver_id, DriverState::Faulted);
                update_driver_state(drv.driver_id, DriverState::Restarting);
                
                // Restart it
                let new_pid = spawn(drv.name);
                drv.pid = new_pid;
                
                // Rebind it
                bind_driver("ahci_driver", drv.device_id);
            }
        }
        yield_cpu();
    }
}

fn bind_driver(ipc_name: &str, device_id: u64) {
    let mut handle = 2; // ENOENT
    for _ in 0..1000 {
        handle = find(ipc_name);
        if handle != 2 { break; }
        yield_cpu();
    }
    
    if handle != 2 {
        // Send BindDevice IPC
        let msg = BindDeviceMsg {
            msg_id: IpcMessageId::BindDevice as u8,
            device_id,
        };
        let bytes = unsafe { core::slice::from_raw_parts(&msg as *const _ as *const u8, core::mem::size_of::<BindDeviceMsg>()) };
        send(handle, bytes);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}
