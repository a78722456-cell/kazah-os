use super::{FileSystem, Vnode, FileType, VfsError};
use crate::security::{SecurityDescriptor, IntegrityLevel};
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::collections::BTreeMap;

pub struct TmpFs {
    root: Arc<Vnode>,
    nodes: Mutex<BTreeMap<u64, TmpFsNode>>,
    next_inode: Mutex<u64>,
}

pub enum TmpFsNodeData {
    File(Vec<u8>),
    Directory(BTreeMap<String, u64>),
}

pub struct TmpFsNode {
    pub data: TmpFsNodeData,
}

impl TmpFs {
    pub fn new() -> Arc<Self> {
        let fs = Arc::new(Self {
            root: unsafe { core::mem::MaybeUninit::uninit().assume_init() }, // Placeholder, fixed below
            nodes: Mutex::new(BTreeMap::new()),
            next_inode: Mutex::new(2),
        });

        // Initialize root directory
        let root_inode = 1;
        fs.nodes.lock().insert(root_inode, TmpFsNode {
            data: TmpFsNodeData::Directory(BTreeMap::new()),
        });

        // Cast pointer to mutate root (safe during init)
        let fs_ptr = Arc::into_raw(fs.clone()) as *mut Self;
        unsafe {
            (*fs_ptr).root = Arc::new(Vnode {
                fs: fs.clone(),
                file_type: FileType::Directory,
                inode_num: root_inode,
                size: Mutex::new(0),
                security: Mutex::new(SecurityDescriptor::system_default()),
                internal_data: 0,
            });
        }
        let _ = unsafe { Arc::from_raw(fs_ptr) };

        fs
    }
    
    fn allocate_inode(&self) -> u64 {
        let mut next = self.next_inode.lock();
        let val = *next;
        *next += 1;
        val
    }
}

impl FileSystem for TmpFs {
    fn root(&self) -> Arc<Vnode> {
        self.root.clone()
    }

    fn lookup(&self, parent: &Arc<Vnode>, name: &str) -> Option<Arc<Vnode>> {
        let nodes = self.nodes.lock();
        if let Some(TmpFsNode { data: TmpFsNodeData::Directory(entries), .. }) = nodes.get(&parent.inode_num) {
            if let Some(&inode_num) = entries.get(name) {
                // Find target node to determine type
                let target_node = nodes.get(&inode_num)?;
                let file_type = match target_node.data {
                    TmpFsNodeData::File(_) => FileType::File,
                    TmpFsNodeData::Directory(_) => FileType::Directory,
                };
                
                let size = match &target_node.data {
                    TmpFsNodeData::File(data) => data.len() as u64,
                    TmpFsNodeData::Directory(_) => 0,
                };

                return Some(Arc::new(Vnode {
                    // Safe because Vnode doesn't actually need exclusive ownership of TmpFs
                    fs: parent.fs.clone(),
                    file_type,
                    inode_num,
                    size: Mutex::new(size),
                    security: Mutex::new(SecurityDescriptor::system_default()), // Ideally stored in TmpFsNode
                    internal_data: 0,
                }));
            }
        }
        None
    }

    fn create(&self, parent: &Arc<Vnode>, name: &str, file_type: FileType, _security: SecurityDescriptor) -> Result<Arc<Vnode>, VfsError> {
        let mut nodes = self.nodes.lock();
        
        // Ensure parent is directory
        let parent_node = nodes.get_mut(&parent.inode_num).ok_or(VfsError::NotFound)?;
        if let TmpFsNodeData::Directory(entries) = &mut parent_node.data {
            if entries.contains_key(name) {
                return Err(VfsError::AlreadyExists);
            }
            
            let new_inode = self.allocate_inode();
            entries.insert(String::from(name), new_inode);
            
            let data = match file_type {
                FileType::File => TmpFsNodeData::File(Vec::new()),
                FileType::Directory => TmpFsNodeData::Directory(BTreeMap::new()),
                _ => return Err(VfsError::Unsupported),
            };
            
            nodes.insert(new_inode, TmpFsNode { data });
            
            Ok(Arc::new(Vnode {
                fs: parent.fs.clone(),
                file_type,
                inode_num: new_inode,
                size: Mutex::new(0),
                security: Mutex::new(SecurityDescriptor::system_default()),
                internal_data: 0,
            }))
        } else {
            Err(VfsError::NotADirectory)
        }
    }

    fn mkdir(&self, parent: &Arc<Vnode>, name: &str, security: SecurityDescriptor) -> Result<Arc<Vnode>, VfsError> {
        self.create(parent, name, FileType::Directory, security)
    }

    fn read(&self, vnode: &Arc<Vnode>, offset: u64, buf: &mut [u8]) -> Result<usize, VfsError> {
        let nodes = self.nodes.lock();
        if let Some(TmpFsNode { data: TmpFsNodeData::File(data), .. }) = nodes.get(&vnode.inode_num) {
            let offset = offset as usize;
            if offset >= data.len() {
                return Ok(0);
            }
            let read_len = core::cmp::min(buf.len(), data.len() - offset);
            buf[..read_len].copy_from_slice(&data[offset..offset + read_len]);
            Ok(read_len)
        } else {
            Err(VfsError::IsADirectory)
        }
    }

    fn write(&self, vnode: &Arc<Vnode>, offset: u64, buf: &[u8]) -> Result<usize, VfsError> {
        let mut nodes = self.nodes.lock();
        if let Some(TmpFsNode { data: TmpFsNodeData::File(data), .. }) = nodes.get_mut(&vnode.inode_num) {
            let offset = offset as usize;
            if offset + buf.len() > data.len() {
                data.resize(offset + buf.len(), 0);
            }
            data[offset..offset + buf.len()].copy_from_slice(buf);
            *vnode.size.lock() = data.len() as u64;
            Ok(buf.len())
        } else {
            Err(VfsError::IsADirectory)
        }
    }

    fn read_dir(&self, vnode: &Arc<Vnode>) -> Result<Vec<String>, VfsError> {
        let nodes = self.nodes.lock();
        if let Some(TmpFsNode { data: TmpFsNodeData::Directory(entries), .. }) = nodes.get(&vnode.inode_num) {
            Ok(entries.keys().cloned().collect())
        } else {
            Err(VfsError::NotADirectory)
        }
    }
}
