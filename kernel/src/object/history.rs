use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::object::ObjectType;
use crate::object::process::ProcessSnapshot;
use crate::object::device::DeviceSnapshot;

use crate::object::driver::DriverSnapshot;
use crate::object::storage::StorageSnapshot;

#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub timestamp: u64,
    pub object_id: u64,
    pub object_type: ObjectType,
    pub revision: u64,
}

#[derive(Debug, Clone)]
pub enum SnapshotPayload {
    Process(ProcessSnapshot),
    Device(DeviceSnapshot),
    Driver(DriverSnapshot),
    Storage(StorageSnapshot),
    // Future additions for Volume, EventBus, Window, etc.
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub header: SnapshotHeader,
    pub payload: SnapshotPayload,
}

lazy_static! {
    pub static ref HISTORIAN: Mutex<VecDeque<Snapshot>> = Mutex::new(VecDeque::new());
}
