use alloc::collections::{VecDeque, BTreeMap};
use alloc::string::String;
use spin::Mutex;
use crate::object::{KernelObject, ObjectHeader, ObjectType};
use crate::object::process::ProcessId;
use lazy_static::lazy_static;

pub struct ServiceRegistration {
    pub pid: ProcessId,
    pub channel: alloc::sync::Arc<ChannelObject>,
    pub start_time: u64,
    pub restart_count: u32,
    pub singleton: bool,
}

lazy_static! {
    pub static ref REGISTRY: Mutex<BTreeMap<String, ServiceRegistration>> = Mutex::new(BTreeMap::new());
}

pub const IPC_MAX_MESSAGE_SIZE: usize = 256;

#[derive(Clone, Copy)]
pub struct Message {
    pub len: u16,
    pub data: [u8; IPC_MAX_MESSAGE_SIZE],
}

pub struct ChannelObject {
    pub header: ObjectHeader,
    pub owner_pid: ProcessId,
    pub queue: Mutex<VecDeque<Message>>,
}

impl KernelObject for ChannelObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl ChannelObject {
    pub fn new(owner_pid: ProcessId) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Channel),
            owner_pid,
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn send(&self, msg: Message) {
        self.queue.lock().push_back(msg);
    }

    pub fn try_recv(&self) -> Option<Message> {
        self.queue.lock().pop_front()
    }
}
