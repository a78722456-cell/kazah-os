use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use crate::object::KernelObject;
use spin::Mutex;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HandleRights: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const DUPLICATE = 1 << 2;
        const ALL = Self::READ.bits() | Self::WRITE.bits() | Self::DUPLICATE.bits();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle(pub u64);

pub struct HandleEntry {
    pub object: Arc<dyn KernelObject + Send + Sync>,
    pub rights: HandleRights,
}

use core::sync::atomic::{AtomicU64, Ordering};

pub struct HandleTable {
    next_handle: AtomicU64,
    entries: Mutex<BTreeMap<Handle, HandleEntry>>,
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(1), // 0 is often reserved or invalid
            entries: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn add(&self, object: Arc<dyn KernelObject + Send + Sync>, rights: HandleRights) -> Handle {
        let mut entries = self.entries.lock();
        let handle_val = self.next_handle.fetch_add(1, Ordering::SeqCst);
        let handle = Handle(handle_val);
        
        entries.insert(handle, HandleEntry { object, rights });
        
        handle
    }

    pub fn get(&self, handle: Handle) -> Option<HandleEntry> {
        self.entries.lock().get(&handle).map(|entry| HandleEntry {
            object: entry.object.clone(),
            rights: entry.rights,
        })
    }

    pub fn remove(&self, handle: Handle) -> Option<HandleEntry> {
        self.entries.lock().remove(&handle)
    }
}
