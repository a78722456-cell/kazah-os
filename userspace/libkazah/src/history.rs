use crate::process::ProcessSnapshot;
use crate::device::DeviceSnapshot;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ObjectType {
    Process = 0,
    Thread = 1,
    Channel = 2,
    Event = 3,
    Mutex = 4,
    Semaphore = 5,
    Timer = 6,
    Device = 7,
    File = 8,
}

#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub timestamp: u64,
    pub object_id: u64,
    pub object_type: ObjectType,
}

#[derive(Debug, Clone)]
pub enum SnapshotRecord {
    Process(ProcessSnapshot),
    Device(DeviceSnapshot),
}
