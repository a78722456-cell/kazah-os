use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;
use crate::memory::address_space::AddressSpace;
use crate::object::handle::HandleTable;
use crate::object::thread::ThreadObject;
use crate::object::{KernelObject, ObjectHeader, ObjectType};
use core::sync::atomic::{AtomicU64, Ordering};
use crate::object::history::SnapshotHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u64);

static NEXT_PID: AtomicU64 = AtomicU64::new(1);

impl ProcessId {
    pub fn new() -> Self {
        ProcessId(NEXT_PID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Starting,
    Running,
    Waiting,
    Stopped,
    Crashed,
    Zombie,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessExit {
    pub state: ProcessState,
    pub code: i32,
}

#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub state: ProcessState,
    pub uptime_ticks: u64,
    pub restart_count: u32,
    pub open_handles_count: usize,
    pub thread_count: usize,
    pub cr3_address: u64,
    // Using a fixed size array to avoid allocation complexities across boundaries
    pub image_name: [u8; 64],
}

pub struct BootstrapContext {
    pub service_manager: Option<crate::object::handle::Handle>,
}

pub struct ProcessObject {
    pub header: ObjectHeader,
    pub id: ProcessId,
    pub address_space: AddressSpace,
    pub handle_table: HandleTable,
    pub threads: Mutex<Vec<Arc<ThreadObject>>>,
    pub token: Arc<crate::security::SecurityToken>,
    pub bootstrap_ctx: Mutex<BootstrapContext>,
    
    // Phase 8.2
    pub state: Mutex<ProcessState>,
    pub exit_info: Mutex<Option<ProcessExit>>,
    pub image_name: [u8; 64],
    pub restart_count: AtomicU64,
    pub start_time: u64,
}

impl KernelObject for ProcessObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl ProcessObject {
    pub fn new(is_root: bool, name: &str) -> Option<Self> {
        let mut image_name = [0u8; 64];
        let len = core::cmp::min(name.len(), 64);
        image_name[..len].copy_from_slice(&name.as_bytes()[..len]);

        let token = if is_root {
            Arc::new(crate::security::SecurityToken::system())
        } else {
            // Default user token
            Arc::new(crate::security::SecurityToken::new(
                1000, 1000, 
                crate::security::IntegrityLevel::Medium, 
                crate::security::Capabilities::empty()
            ))
        };

        Some(Self {
            header: ObjectHeader::new(ObjectType::Process),
            id: ProcessId::new(),
            address_space: AddressSpace::new()?,
            handle_table: HandleTable::new(),
            threads: Mutex::new(Vec::new()),
            token,
            bootstrap_ctx: Mutex::new(crate::object::process::BootstrapContext { service_manager: None }),
            state: Mutex::new(ProcessState::Starting),
            exit_info: Mutex::new(None),
            image_name,
            restart_count: core::sync::atomic::AtomicU64::new(0),
            start_time: 0,
        })
    }

    pub fn snapshot(&self) -> crate::object::history::Snapshot {
        crate::object::history::Snapshot {
            header: SnapshotHeader {
                timestamp: 0,
                object_id: self.id.0,
                object_type: ObjectType::Process,
                revision: self.header.version.load(Ordering::Relaxed),
            },
            payload: crate::object::history::SnapshotPayload::Process(ProcessSnapshot {
                state: *self.state.lock(),
                uptime_ticks: 0, // TODO: timer
                restart_count: self.restart_count.load(Ordering::Relaxed) as u32,
                open_handles_count: 0, // TODO: expose handle_table count
                thread_count: self.threads.lock().len(),
                cr3_address: self.address_space.cr3.as_u64(),
                image_name: self.image_name,
            }),
        }
    }

    pub fn add_thread(&self, thread: Arc<ThreadObject>) {
        self.threads.lock().push(thread);
    }
}
