#![no_std]
#![no_main]

use libkazah::syscalls::{exit, get_pci_devices, update_device_state};
use libkazah::device::{PciDeviceInfo, DeviceState};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut devices = [PciDeviceInfo {
        id: 0, bus: 0, device: 0, function: 0, vendor_id: 0, device_id: 0, class_code: 0, subclass: 0, prog_if: 0,
    }; 32];
    
    let count = get_pci_devices(&mut devices);
    if count > 0 {
        let dev = devices[0].id;
        
        for _ in 0..1000 {
            update_device_state(dev, DeviceState::Found, None);
            update_device_state(dev, DeviceState::Registered, None);
            update_device_state(dev, DeviceState::DriverLoaded, None);
            update_device_state(dev, DeviceState::Ready, None);
            update_device_state(dev, DeviceState::Lost, None);
        }
    }
    
    exit(0);
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    libkazah::syscalls::exit(-1);
}
