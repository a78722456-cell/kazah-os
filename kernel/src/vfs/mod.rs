pub mod mount;
pub mod tmpfs;
pub mod devfs;

use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::object::{KernelObject, ObjectHeader, ObjectType};
use crate::security::SecurityDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
}

pub trait FileSystem: Send + Sync {
    fn root(&self) -> Arc<Vnode>;
    fn lookup(&self, parent: &Arc<Vnode>, name: &str) -> Option<Arc<Vnode>>;
    fn create(&self, parent: &Arc<Vnode>, name: &str, file_type: FileType, security: SecurityDescriptor) -> Result<Arc<Vnode>, VfsError>;
    fn mkdir(&self, parent: &Arc<Vnode>, name: &str, security: SecurityDescriptor) -> Result<Arc<Vnode>, VfsError>;
    fn read(&self, vnode: &Arc<Vnode>, offset: u64, buf: &mut [u8]) -> Result<usize, VfsError>;
    fn write(&self, vnode: &Arc<Vnode>, offset: u64, buf: &[u8]) -> Result<usize, VfsError>;
    fn read_dir(&self, vnode: &Arc<Vnode>) -> Result<Vec<String>, VfsError>;
}

#[derive(Debug)]
pub enum VfsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    PermissionDenied,
    NoSpace,
    Unsupported,
}

pub struct Vnode {
    pub fs: Arc<dyn FileSystem>,
    pub file_type: FileType,
    pub inode_num: u64,
    pub size: Mutex<u64>,
    pub security: Mutex<SecurityDescriptor>,
    pub internal_data: u64, // Filesystem specific
}

pub struct FileObject {
    pub header: ObjectHeader,
    pub vnode: Arc<Vnode>,
    pub cursor: Mutex<u64>,
    pub mode: u32, // Read/Write flags
}

impl KernelObject for FileObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl FileObject {
    pub fn new(vnode: Arc<Vnode>, mode: u32) -> Self {
        Self {
            header: ObjectHeader::new(ObjectType::File),
            vnode,
            cursor: Mutex::new(0),
            mode,
        }
    }
}
