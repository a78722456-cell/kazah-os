pub trait BlockDevice {
    fn read_blocks(&self, lba: u64, count: u32, buffer: &mut [u8]) -> i32;
    fn write_blocks(&self, lba: u64, count: u32, buffer: &[u8]) -> i32;
    fn flush(&self) -> i32;
    fn block_size(&self) -> u32;
    fn block_count(&self) -> u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageOpcode {
    Read = 1,
    Write = 2,
    Flush = 3,
    Identify = 4,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StorageRequestHeader {
    pub opcode: u8,
    pub request_id: u64,
    pub lba: u64,
    pub count: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StorageResponseHeader {
    pub request_id: u64,
    pub status: i32, // 0 = Success, < 0 = Error code
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StorageIdentifyInfo {
    pub block_size: u32,
    pub block_count: u64,
}

// Volume Opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VolumeOpcode {
    Create = 10,
    Delete = 11,
    PartitionRead = 12,
    PartitionWrite = 13,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VolumeRequestHeader {
    pub opcode: u8,
    pub request_id: u64,
    pub partition_id: u64,
    pub lba: u64,
    pub count: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VolumeResponseHeader {
    pub request_id: u64,
    pub status: i32,
}
