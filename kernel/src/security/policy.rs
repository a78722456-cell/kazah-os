use super::{SecurityToken, SecurityDescriptor, AccessType, AceType, IntegrityLevel, Capabilities};

#[derive(Debug)]
pub enum AccessError {
    AccessDenied,
    IntegrityLevelTooLow,
    MissingCapability,
}

pub struct AccessManager;

impl AccessManager {
    pub fn check_access(token: &SecurityToken, descriptor: &SecurityDescriptor, desired_access: u32) -> Result<(), AccessError> {
        // 1. Mandatory Access Control (MAC) - Integrity Levels
        // Rule: "No write up"
        let is_write_intent = (desired_access & (AccessType::Write as u32 | AccessType::Delete as u32 | AccessType::ChangeAcl as u32 | AccessType::ChangeOwner as u32)) != 0;
        
        if is_write_intent && token.integrity_level < descriptor.mandatory_integrity_level {
            // Bypass if process has SECURITY_ADMIN and is trying to change ACL
            if !(token.capabilities.contains(Capabilities::CAP_SECURITY_ADMIN) && 
                (desired_access & (AccessType::ChangeAcl as u32 | AccessType::ChangeOwner as u32)) != 0) {
                return Err(AccessError::IntegrityLevelTooLow);
            }
        }

        // 2. Discretionary Access Control (DAC) - ACLs
        // Owner always has full access (implicit, though can be restricted by explicit deny)
        // Wait, standard practice: Evaluate Explicit Deny, then Explicit Allow.
        
        let mut granted_access = 0;
        
        // If owner, grant full access by default, but still evaluate ACLs in case of explicit deny
        if token.uid == descriptor.owner_uid {
            granted_access = 0xFFFFFFFF;
        }

        for ace in &descriptor.acl {
            let match_uid = ace.uid.map_or(false, |uid| uid == token.uid);
            let match_gid = ace.gid.map_or(false, |gid| gid == token.gid || token.supplementary_gids.contains(&gid));
            let match_all = ace.uid.is_none() && ace.gid.is_none(); // "Everyone" ACE

            if match_uid || match_gid || match_all {
                if ace.ace_type == AceType::Deny {
                    if (desired_access & ace.access_mask) != 0 {
                        // Explicit deny matched
                        crate::security::audit::log_denial(token, descriptor, desired_access, "Explicit Deny ACE matched");
                        return Err(AccessError::AccessDenied);
                    }
                } else if ace.ace_type == AceType::Allow {
                    granted_access |= ace.access_mask;
                }
            }
        }

        if (desired_access & granted_access) == desired_access {
            Ok(())
        } else {
            // Cap check for fallback
            if token.capabilities.contains(Capabilities::CAP_SYS_ADMIN) {
                return Ok(()); // Root equivalent
            }
            crate::security::audit::log_denial(token, descriptor, desired_access, "Insufficient DAC rights");
            Err(AccessError::AccessDenied)
        }
    }
}
