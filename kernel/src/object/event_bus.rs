use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    StorageDeviceConnected,
    StorageDeviceDisconnected,
    VolumeMounted,
    VolumeUnmounted,
    DriverLoaded,
    DriverUnloaded,
    ObjectCreated,
    ObjectDestroyed,
}

#[derive(Debug, Clone)]
pub struct SystemEvent {
    pub event_type: EventType,
    pub timestamp: u64,
    pub object_id: u64,
    pub payload_data: [u8; 32], // Small inline payload
}

pub struct EventBus {
    pub events: Mutex<VecDeque<SystemEvent>>,
    // In a real system, we would have subscribers (e.g. IPC channels to notify)
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn publish(&self, event: SystemEvent) {
        let mut queue = self.events.lock();
        queue.push_back(event);
        if queue.len() > 1024 {
            queue.pop_front();
        }
        // TODO: Wake up subscribers
    }
    
    pub fn pull(&self) -> Option<SystemEvent> {
        self.events.lock().pop_front()
    }
}

lazy_static! {
    pub static ref GLOBAL_EVENT_BUS: Arc<EventBus> = Arc::new(EventBus::new());
}
