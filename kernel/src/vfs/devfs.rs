use super::{FileSystem, Vnode, FileType, VfsError};
use crate::security::SecurityDescriptor;
use crate::object::storage::STORAGE_REGISTRY;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

pub struct DevFs {
    root: Arc<Vnode>,
}

impl DevFs {
    pub fn new() -> Arc<Self> {
        let fs = Arc::new(Self {
            root: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
        });

        let fs_ptr = Arc::into_raw(fs.clone()) as *mut Self;
        unsafe {
            (*fs_ptr).root = Arc::new(Vnode {
                fs: fs.clone(),
                file_type: FileType::Directory,
                inode_num: 1, // Root is 1
                size: Mutex::new(0),
                security: Mutex::new(SecurityDescriptor::system_default()),
                internal_data: 0,
            });
        }
        let _ = unsafe { Arc::from_raw(fs_ptr) };

        fs
    }
}

impl FileSystem for DevFs {
    fn root(&self) -> Arc<Vnode> {
        self.root.clone()
    }

    fn lookup(&self, parent: &Arc<Vnode>, name: &str) -> Option<Arc<Vnode>> {
        if parent.inode_num != 1 {
            return None; // DevFS has no subdirectories for now
        }

        // Search through the storage registry
        let registry = STORAGE_REGISTRY.lock();
        for storage in registry.iter() {
            // Very simple name generation, e.g., "sda", "ramdisk0"
            let storage_name = alloc::format!("storage{}", storage.id);
            if name == storage_name {
                return Some(Arc::new(Vnode {
                    fs: parent.fs.clone(),
                    file_type: FileType::BlockDevice,
                    inode_num: storage.id, // Use Object ID as inode
                    size: Mutex::new(storage.capacity),
                    security: Mutex::new(SecurityDescriptor::system_default()),
                    internal_data: storage.id, // Save object ID
                }));
            }
        }
        None
    }

    fn create(&self, _parent: &Arc<Vnode>, _name: &str, _file_type: FileType, _security: SecurityDescriptor) -> Result<Arc<Vnode>, VfsError> {
        Err(VfsError::Unsupported) // Can't create files in /dev directly
    }

    fn mkdir(&self, _parent: &Arc<Vnode>, _name: &str, _security: SecurityDescriptor) -> Result<Arc<Vnode>, VfsError> {
        Err(VfsError::Unsupported)
    }

    fn read(&self, vnode: &Arc<Vnode>, offset: u64, buf: &mut [u8]) -> Result<usize, VfsError> {
        // Find the storage object and read
        let storage_id = vnode.internal_data;
        let registry = STORAGE_REGISTRY.lock();
        if let Some(storage) = registry.iter().find(|s| s.id == storage_id) {
            // Assuming storage provides direct read. In a real OS, we send an IPC message to the storage driver.
            // For now, return Unsupported since IPC must be used.
            // Ideally VFS -> IPC -> Driver
            return Err(VfsError::Unsupported);
        }
        Err(VfsError::NotFound)
    }

    fn write(&self, _vnode: &Arc<Vnode>, _offset: u64, _buf: &[u8]) -> Result<usize, VfsError> {
        Err(VfsError::Unsupported)
    }

    fn read_dir(&self, parent: &Arc<Vnode>) -> Result<Vec<String>, VfsError> {
        if parent.inode_num != 1 {
            return Err(VfsError::NotADirectory);
        }
        
        let mut entries = Vec::new();
        let registry = STORAGE_REGISTRY.lock();
        for storage in registry.iter() {
            entries.push(alloc::format!("storage{}", storage.id));
        }
        Ok(entries)
    }
}
