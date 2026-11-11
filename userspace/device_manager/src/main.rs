#![no_std]
#![no_main]

use libkazah::syscalls::{exit, register, get_pci_devices, update_device_state};
use libkazah::device::{PciDeviceInfo, DeviceState};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let chan = register("device_manager");
    
    // 1. Get PCI Devices
    let mut devices = [PciDeviceInfo {
        id: 0, bus: 0, device: 0, function: 0, vendor_id: 0, device_id: 0, class_code: 0, subclass: 0, prog_if: 0,
    }; 32];
    
    let count = get_pci_devices(&mut devices);
    
    // 2. Register devices
    for i in 0..count as usize {
        let dev = &devices[i];
        let mut name = [0u8; 16];
        let prefix = b"pci-dev";
        name[..7].copy_from_slice(prefix);
        
        update_device_state(dev.id, DeviceState::Registered, Some(&name));
    }
    
    // 3. Monitor (stub)
    loop {
        libkazah::syscalls::yield_cpu();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    libkazah::syscalls::exit(-1);
}
