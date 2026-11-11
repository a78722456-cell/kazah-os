use super::{SecurityToken, SecurityDescriptor};
use alloc::format;

// Basic audit log stub. In a full system, this would write to the EventBus or a specialized Audit Historian queue.

pub fn log_denial(token: &SecurityToken, _descriptor: &SecurityDescriptor, desired_access: u32, reason: &str) {
    let msg = format!(
        "[AUDIT DENY] UID: {}, IL: {:?}, Access Requested: 0x{:X}, Reason: {}",
        token.uid, token.integrity_level, desired_access, reason
    );
    crate::println!("{}", msg);
    // TODO: Publish to EventBus
}

pub fn log_success(token: &SecurityToken, _descriptor: &SecurityDescriptor, desired_access: u32, action: &str) {
    let msg = format!(
        "[AUDIT SUCCESS] UID: {}, IL: {:?}, Access Granted: 0x{:X}, Action: {}",
        token.uid, token.integrity_level, desired_access, action
    );
    // For performance, success audits are usually disabled or filtered by policy.
    crate::println!("{}", msg);
}
