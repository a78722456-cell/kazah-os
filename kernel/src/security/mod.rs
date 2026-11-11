pub mod policy;
pub mod audit;
pub mod signing;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum IntegrityLevel {
    Untrusted = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    System = 4,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Capabilities: u64 {
        const CAP_SYS_ADMIN      = 1 << 0;
        const CAP_MOUNT          = 1 << 1;
        const CAP_DRIVER_LOAD    = 1 << 2;
        const CAP_IPC_MONITOR    = 1 << 3;
        const CAP_SECURITY_ADMIN = 1 << 4;
        const CAP_KILL           = 1 << 5;
    }
}

#[derive(Debug, Clone)]
pub struct SecurityToken {
    pub token_id: u64,
    pub uid: u32,
    pub gid: u32,
    pub supplementary_gids: Vec<u32>,
    pub integrity_level: IntegrityLevel,
    pub capabilities: Capabilities,
}

static NEXT_TOKEN_ID: AtomicU64 = AtomicU64::new(1);

impl SecurityToken {
    pub fn new(uid: u32, gid: u32, integrity_level: IntegrityLevel, capabilities: Capabilities) -> Self {
        Self {
            token_id: NEXT_TOKEN_ID.fetch_add(1, Ordering::Relaxed),
            uid,
            gid,
            supplementary_gids: Vec::new(),
            integrity_level,
            capabilities,
        }
    }

    pub fn system() -> Self {
        Self::new(0, 0, IntegrityLevel::System, Capabilities::all())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccessType {
    Read = 1,
    Write = 2,
    Execute = 4,
    Delete = 8,
    ChangeOwner = 16,
    ChangeAcl = 32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AceType {
    Allow = 0,
    Deny = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessControlEntry {
    pub ace_type: AceType,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub access_mask: u32,
}

#[derive(Debug, Clone)]
pub struct SecurityDescriptor {
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub acl: Vec<AccessControlEntry>,
    pub mandatory_integrity_level: IntegrityLevel,
}

impl SecurityDescriptor {
    pub fn new(owner_uid: u32, owner_gid: u32, mandatory_integrity_level: IntegrityLevel) -> Self {
        Self {
            owner_uid,
            owner_gid,
            acl: Vec::new(),
            mandatory_integrity_level,
        }
    }

    pub fn system_default() -> Self {
        Self::new(0, 0, IntegrityLevel::System)
    }

    pub fn user_default(uid: u32, gid: u32) -> Self {
        let mut sd = Self::new(uid, gid, IntegrityLevel::Medium);
        sd.acl.push(AccessControlEntry {
            ace_type: AceType::Allow,
            uid: Some(uid),
            gid: None,
            access_mask: 0xFFFFFFFF, // Full access
        });
        sd
    }
}
