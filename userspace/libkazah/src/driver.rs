#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DriverState {
    Loading = 0,
    Binding = 1,
    Running = 2,
    Restarting = 3,
    Faulted = 4,
    Unloaded = 5,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IpcMessageId {
    BindDevice = 1,
    CrashDriver = 2,
    UnbindDevice = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BindDeviceMsg {
    pub msg_id: u8,
    pub device_id: u64,
}
