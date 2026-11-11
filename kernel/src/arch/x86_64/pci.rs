use x86_64::instructions::port::Port;
use crate::object::device::{PciDevice, DeviceObject, DEVICE_REGISTRY};
use alloc::sync::Arc;

pub fn config_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
        
    let mut config_address = Port::<u32>::new(0xCF8);
    let mut config_data = Port::<u32>::new(0xCFC);
    
    unsafe {
        config_address.write(address);
        config_data.read()
    }
}

pub fn config_read_u16(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let data = config_read_u32(bus, slot, func, offset);
    ((data >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

pub fn config_read_u8(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    let data = config_read_u32(bus, slot, func, offset);
    ((data >> ((offset & 3) * 8)) & 0xFF) as u8
}

pub fn enumerate_pci() {
    let mut registry = DEVICE_REGISTRY.lock();
    
    for bus in 0..=255 {
        for slot in 0..32 {
            let vendor_id = config_read_u16(bus, slot, 0, 0);
            if vendor_id == 0xFFFF {
                continue; // Device doesn't exist
            }
            
            // Check if multi-function
            let header_type = config_read_u8(bus, slot, 0, 0x0E);
            let multi_func = (header_type & 0x80) != 0;
            let functions = if multi_func { 8 } else { 1 };
            
            for func in 0..functions {
                let vendor_id = config_read_u16(bus, slot, func, 0);
                if vendor_id == 0xFFFF {
                    continue;
                }
                
                let device_id = config_read_u16(bus, slot, func, 2);
                let class_code = config_read_u8(bus, slot, func, 0x0B);
                let subclass = config_read_u8(bus, slot, func, 0x0A);
                let prog_if = config_read_u8(bus, slot, func, 0x09);
                
                let pci_dev = PciDevice {
                    bus,
                    device: slot,
                    function: func,
                    vendor_id,
                    device_id,
                    class_code,
                    subclass,
                    prog_if,
                };
                
                let dev_obj = Arc::new(DeviceObject::new(pci_dev));
                registry.push(dev_obj);
            }
        }
    }
}
