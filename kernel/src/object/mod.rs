pub mod thread;
pub mod process;
pub mod handle;
pub mod ipc;
pub mod history;
pub mod device;
pub mod driver;
pub mod storage;
pub mod storage_fw;
pub mod event_bus;

use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Process,
    Thread,
    Channel,
    Event,
    Mutex,
    Semaphore,
    Timer,
    Device,
    File,
    Driver,
    Storage,
    Disk,
    Partition,
    Volume,
    SnapshotVolume,
}

pub struct ObjectHeader {
    pub type_id: ObjectType,
    pub id: u64,
    pub version: core::sync::atomic::AtomicU64,
    pub security: crate::security::SecurityDescriptor,
    pub handle_count: AtomicUsize,
}

static NEXT_OBJECT_ID: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);

pub fn next_object_id() -> u64 {
    NEXT_OBJECT_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

impl ObjectHeader {
    pub fn new(type_id: ObjectType) -> Self {
        Self {
            type_id,
            id: next_object_id(),
            version: core::sync::atomic::AtomicU64::new(1),
            security: crate::security::SecurityDescriptor::system_default(),
            handle_count: AtomicUsize::new(0),
        }
    }
}

use core::any::Any;

pub trait KernelObject: Any + Send + Sync {
    fn header(&self) -> &ObjectHeader;
    fn as_any(&self) -> &dyn Any;
}
