use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::object::{KernelObject, ObjectHeader, ObjectType};
use crate::object::history::SnapshotHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DriverState {
    Loading = 0,
    Binding = 1,
    Running = 2,
    Restarting = 3,
    Faulted = 4,
    Unloaded = 5,
    Registered = 6,
}

#[derive(Debug, Clone)]
pub struct DriverSnapshot {
    pub driver_name: [u8; 16],
    pub device_id: u64,
    pub restart_count: u64,
    pub uptime_ticks: u64,
    pub state: DriverState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverCapabilities {
    pub dma: bool,
    pub hotplug: bool,
    pub power_management: bool,
    pub interrupts: bool,
    pub msi: bool,
    pub msix: bool,
}

impl Default for DriverCapabilities {
    fn default() -> Self {
        Self {
            dma: false,
            hotplug: false,
            power_management: false,
            interrupts: false,
            msi: false,
            msix: false,
        }
    }
}

pub struct DriverObject {
    pub header: ObjectHeader,
    pub id: u64,
    pub name: String,
    pub device_id: u64, // The device it manages
    pub service_handle: Option<u64>, // Process/service handling it
    pub restart_count: Mutex<u64>,
    pub state: Mutex<DriverState>,
}

impl KernelObject for DriverObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl DriverObject {
    pub fn new(id: u64, name: String, device_id: u64) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Driver),
            id,
            name,
            device_id,
            service_handle: None,
            restart_count: Mutex::new(0),
            state: Mutex::new(DriverState::Registered),
        }
    }
    
    pub fn snapshot(&self) -> crate::object::history::Snapshot {
        let mut name_buf = [0u8; 16];
        let name_bytes = self.name.as_bytes();
        let copy_len = core::cmp::min(16, name_bytes.len());
        name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        crate::object::history::Snapshot {
            header: SnapshotHeader {
                timestamp: 0, // TODO: timer
                object_id: self.id,
                object_type: ObjectType::Driver,
                revision: self.header.version.load(core::sync::atomic::Ordering::Relaxed),
            },
            payload: crate::object::history::SnapshotPayload::Driver(DriverSnapshot {
                driver_name: name_buf,
                device_id: self.device_id,
                restart_count: *self.restart_count.lock(),
                uptime_ticks: 0,
                state: *self.state.lock(),
            }),
        }
    }
}

lazy_static! {
    pub static ref DRIVER_REGISTRY: Mutex<Vec<Arc<DriverObject>>> = Mutex::new(Vec::new());
}
