use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::object::{KernelObject, ObjectHeader, ObjectType};

pub struct DiskObject {
    pub header: ObjectHeader,
    pub storage_id: u64, // Maps to the underlying StorageObject block device
    pub total_blocks: u64,
    pub block_size: u32,
    pub is_removable: bool,
}

impl KernelObject for DiskObject {
    fn header(&self) -> &ObjectHeader { &self.header }
    fn as_any(&self) -> &dyn core::any::Any { self }
}

impl DiskObject {
    pub fn new(storage_id: u64, total_blocks: u64, block_size: u32, is_removable: bool) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Disk),
            storage_id,
            total_blocks,
            block_size,
            is_removable,
        }
    }
}

pub struct PartitionObject {
    pub header: ObjectHeader,
    pub disk_id: u64,
    pub start_lba: u64,
    pub length_lba: u64,
    pub partition_type: [u8; 16],
}

impl KernelObject for PartitionObject {
    fn header(&self) -> &ObjectHeader { &self.header }
    fn as_any(&self) -> &dyn core::any::Any { self }
}

impl PartitionObject {
    pub fn new(disk_id: u64, start_lba: u64, length_lba: u64, partition_type: [u8; 16]) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Partition),
            disk_id,
            start_lba,
            length_lba,
            partition_type,
        }
    }
}

pub struct VolumeObject {
    pub header: ObjectHeader,
    pub partition_id: Option<u64>,
    pub disk_id: Option<u64>,
    pub start_lba: u64,
    pub length_lba: u64,
    pub is_mounted: Mutex<bool>,
}

impl KernelObject for VolumeObject {
    fn header(&self) -> &ObjectHeader { &self.header }
    fn as_any(&self) -> &dyn core::any::Any { self }
}

impl VolumeObject {
    pub fn new(partition_id: Option<u64>, disk_id: Option<u64>, start_lba: u64, length_lba: u64) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::Volume),
            partition_id,
            disk_id,
            start_lba,
            length_lba,
            is_mounted: Mutex::new(false),
        }
    }
}

lazy_static! {
    pub static ref DISK_REGISTRY: Mutex<Vec<Arc<DiskObject>>> = Mutex::new(Vec::new());
    pub static ref PARTITION_REGISTRY: Mutex<Vec<Arc<PartitionObject>>> = Mutex::new(Vec::new());
    pub static ref VOLUME_REGISTRY: Mutex<Vec<Arc<VolumeObject>>> = Mutex::new(Vec::new());
}

#[repr(C, packed)]
pub struct GptHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub header_crc32: u32,
    pub reserved: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entries_lba: u64,
    pub num_partition_entries: u32,
    pub partition_entry_size: u32,
    pub partition_entries_crc32: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GptPartitionEntry {
    pub partition_type_guid: [u8; 16],
    pub unique_partition_guid: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub attributes: u64,
    pub partition_name: [u16; 36],
}

pub fn parse_gpt(buffer: &[u8]) -> Option<(GptHeader, Vec<GptPartitionEntry>)> {
    if buffer.len() < 512 {
        return None;
    }
    
    let header_ptr = buffer.as_ptr() as *const GptHeader;
    let header = unsafe { core::ptr::read_unaligned(header_ptr) };
    
    if header.signature != 0x5452415020494645 {
        return None; // Not a GPT
    }
    
    Some((header, Vec::new()))
}
