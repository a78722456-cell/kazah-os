use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::object::{KernelObject, ObjectHeader, ObjectType};
use crate::object::history::SnapshotHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(pub u64);

static NEXT_DEV_ID: AtomicU64 = AtomicU64::new(1);

impl DeviceId {
    pub fn new() -> Self {
        DeviceId(NEXT_DEV_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    // Kernel States
    Found,
    Lost,
    Faulted,
    // Logical States (DeviceManager)
    Registered,
    DriverLoaded,
    Ready,
    Suspended,
}

#[derive(Debug, Clone)]
pub struct PciDevice {
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
    pub state: DeviceState,
    pub driver_loaded: bool,
    pub uptime_ticks: u64,
    pub driver_name: [u8; 16],
}

pub struct DeviceObject {
    pub header: ObjectHeader,
    pub id: DeviceId,
    pub hardware_info: PciDevice,
    pub state: Mutex<DeviceState>,
    pub driver_name: Mutex<[u8; 16]>,
}

impl KernelObject for DeviceObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl DeviceObject {
    pub fn new(hardware_info: PciDevice) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Device),
            id: DeviceId::new(),
            hardware_info,
            state: Mutex::new(DeviceState::Found),
            driver_name: Mutex::new([0; 16]),
        }
    }
    
    pub fn snapshot(&self) -> crate::object::history::Snapshot {
        crate::object::history::Snapshot {
            header: SnapshotHeader {
                timestamp: 0,
                object_id: self.id.0,
                object_type: ObjectType::Device,
                revision: self.header.version.load(Ordering::Relaxed),
            },
            payload: crate::object::history::SnapshotPayload::Device(DeviceSnapshot {
                state: *self.state.lock(),
                driver_loaded: self.driver_name.lock()[0] != 0,
                uptime_ticks: 0,
                driver_name: *self.driver_name.lock(),
            }),
        }
    }
}

use alloc::vec::Vec;
lazy_static::lazy_static! {
    pub static ref DEVICE_REGISTRY: Mutex<Vec<Arc<DeviceObject>>> = Mutex::new(Vec::new());
}
