use super::{Vnode, FileSystem, VfsError};
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

pub struct MountPoint {
    pub path: String,
    pub fs: Arc<dyn FileSystem>,
}

lazy_static! {
    pub static ref MOUNT_MANAGER: Mutex<MountManager> = Mutex::new(MountManager::new());
}

pub struct MountManager {
    mounts: Vec<MountPoint>,
}

impl MountManager {
    pub fn new() -> Self {
        Self { mounts: Vec::new() }
    }

    pub fn mount(&mut self, path: &str, fs: Arc<dyn FileSystem>) {
        self.mounts.push(MountPoint {
            path: String::from(path),
            fs,
        });
        // Sort by path length descending so longer paths match first
        self.mounts.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    }

    pub fn resolve_path(&self, path: &str) -> Result<Arc<Vnode>, VfsError> {
        let path = if !path.starts_with('/') {
            // For now, assume absolute if not starting with / (should be error in real OS)
            return Err(VfsError::NotFound);
        } else {
            path
        };

        // Find longest matching mount point
        let mut target_mount = None;
        let mut remaining_path = "";

        for mount in &self.mounts {
            if path.starts_with(&mount.path) {
                target_mount = Some(mount);
                let mut rem = &path[mount.path.len()..];
                if rem.starts_with('/') {
                    rem = &rem[1..];
                }
                remaining_path = rem;
                break;
            }
        }

        let mount = target_mount.ok_or(VfsError::NotFound)?;
        let mut current_vnode = mount.fs.root();

        if remaining_path.is_empty() {
            return Ok(current_vnode);
        }

        for component in remaining_path.split('/') {
            if component.is_empty() || component == "." {
                continue;
            }
            if component == ".." {
                // Not supported yet
                continue;
            }
            current_vnode = mount.fs.lookup(&current_vnode, component).ok_or(VfsError::NotFound)?;
        }

        Ok(current_vnode)
    }
}
