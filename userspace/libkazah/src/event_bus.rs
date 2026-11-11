#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    StorageDeviceConnected = 0,
    StorageDeviceDisconnected = 1,
    VolumeMounted = 2,
    VolumeUnmounted = 3,
    DriverLoaded = 4,
    DriverUnloaded = 5,
    ObjectCreated = 6,
    ObjectDestroyed = 7,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SystemEvent {
    pub event_type: u32,
    pub timestamp: u64,
    pub object_id: u64,
    pub payload_data: [u8; 32],
}

pub fn subscribe_events() -> i32 {
    unsafe {
        let mut ret: i32;
        core::arch::asm!(
            "syscall",
            in("rax") 17, // Syscall 17 for Event Bus Subscribe
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
        ret
    }
}
