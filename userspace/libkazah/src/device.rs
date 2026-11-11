use crate::history::SnapshotHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Found,
    Lost,
    Faulted,
    Registered,
    DriverLoaded,
    Ready,
    Suspended,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PciDeviceInfo {
    pub id: u64,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
}

#[derive(Debug, Clone)]
pub struct DeviceSnapshot {
    pub header: SnapshotHeader,
    pub state: DeviceState,
    pub driver_loaded: bool,
    pub uptime_ticks: u64,
    pub driver_name: [u8; 16],
}
