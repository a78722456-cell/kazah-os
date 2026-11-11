use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use lazy_static::lazy_static;

use crate::object::{KernelObject, ObjectHeader, ObjectType};
use crate::object::history::SnapshotHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageState {
    Uninitialized = 0,
    Ready = 1,
    Busy = 2,
    Error = 3,
}

#[derive(Debug, Clone)]
pub struct StorageSnapshot {
    pub capacity: u64,
    pub reads: u64,
    pub writes: u64,
    pub io_errors: u64,
    pub driver_pid: u64,
    pub state: StorageState,
}

pub struct StorageObject {
    pub header: ObjectHeader,
    pub id: u64,
    pub device_id: u64,
    pub block_size: u32,
    pub capacity: u64,
    pub driver_pid: u64,
    pub state: Mutex<StorageState>,
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub io_errors: AtomicU64,
}

impl StorageObject {
    pub fn new(id: u64, device_id: u64, block_size: u32, capacity: u64, driver_pid: u64) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Storage),
            id,
            device_id,
            block_size,
            capacity,
            driver_pid,
            state: Mutex::new(StorageState::Uninitialized),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            io_errors: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> crate::object::history::Snapshot {
        crate::object::history::Snapshot {
            header: SnapshotHeader {
                timestamp: 0, // TODO: timer ticks
                object_id: self.id,
                object_type: ObjectType::Storage,
                revision: self.header.version.load(Ordering::Relaxed),
            },
            payload: crate::object::history::SnapshotPayload::Storage(StorageSnapshot {
                capacity: self.capacity,
                reads: self.reads.load(Ordering::Relaxed),
                writes: self.writes.load(Ordering::Relaxed),
                io_errors: self.io_errors.load(Ordering::Relaxed),
                driver_pid: self.driver_pid,
                state: *self.state.lock(),
            }),
        }
    }
}

impl KernelObject for StorageObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

lazy_static! {
    pub static ref STORAGE_REGISTRY: Mutex<Vec<Arc<StorageObject>>> = Mutex::new(Vec::new());
}
