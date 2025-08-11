use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissionSystem {
    // Role definitions and their permissions
    role_permissions: HashMap<UserRole, HashSet<Permission>>,
    // User role assignments per company
    user_roles: HashMap<String, HashMap<String, UserRole>>, // company_id -> user_id -> role
    // Custom permissions for specific users (overrides)
    user_custom_permissions: HashMap<String, HashMap<String, HashSet<Permission>>>, // company_id -> user_id -> permissions
    // Permission audit log
    permission_logs: Vec<PermissionAuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    pub resource: ResourceType,
    pub action: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Company,
    User,
    Product,
    SupplyChain,
    Reports,
    Settings,
    Invitations,
    Verification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActionType {
    Create,
    Read,
    Update,
    Delete,
    Approve,
    Invite,
    Transfer,
    Export,
    Manage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditEntry {
    pub id: String,
    pub user_id: String,
    pub company_id: String,
    pub resource: ResourceType,
    pub action: ActionType,
    pub granted: bool,
    pub reason: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub user_id: String,
    pub company_id: String,
    pub role: UserRole,
    pub assigned_by: String,
    pub assigned_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionError {
    UserNotFound,
    CompanyNotFound,
    InvalidRole,
    InsufficientPermissions,
    UserNotMember,
    RoleNotFound,
    PermissionDenied,
    CannotModifyOwner,
}

impl RolePermissionSystem {
    pub fn new() -> Self {
        let mut system = Self {
            role_permissions: HashMap::new(),
            user_roles: HashMap::new(),
            user_custom_permissions: HashMap::new(),
            permission_logs: Vec::new(),
        };

        // Initialize default role permissions
        system.initialize_default_permissions();
        system
    }

    // Initialize the default permission set for each role
    fn initialize_default_permissions(&mut self) {
        // Owner permissions (full access)
        let owner_permissions = vec![
            Permission { resource: ResourceType::Company, action: ActionType::Manage },
            Permission { resource: ResourceType::User, action: ActionType::Manage },
            Permission { resource: ResourceType::Product, action: ActionType::Manage },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Manage },
            Permission { resource: ResourceType::Reports, action: ActionType::Read },
            Permission { resource: ResourceType::Settings, action: ActionType::Manage },
            Permission { resource: ResourceType::Invitations, action: ActionType::Manage },
            Permission { resource: ResourceType::Verification, action: ActionType::Approve },
        ];

        // Manager permissions (most operations except ownership transfer)
        let manager_permissions = vec![
            Permission { resource: ResourceType::Company, action: ActionType::Update },
            Permission { resource: ResourceType::User, action: ActionType::Read },
            Permission { resource: ResourceType::User, action: ActionType::Invite },
            Permission { resource: ResourceType::Product, action: ActionType::Create },
            Permission { resource: ResourceType::Product, action: ActionType::Read },
            Permission { resource: ResourceType::Product, action: ActionType::Update },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Create },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Read },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Update },
            Permission { resource: ResourceType::Reports, action: ActionType::Read },
            Permission { resource: ResourceType::Reports, action: ActionType::Export },
            Permission { resource: ResourceType::Settings, action: ActionType::Read },
            Permission { resource: ResourceType::Invitations, action: ActionType::Create },
        ];

        // Warehouse permissions (product and supply chain focused)
        let warehouse_permissions = vec![
            Permission { resource: ResourceType::Product, action: ActionType::Read },
            Permission { resource: ResourceType::Product, action: ActionType::Update },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Create },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Read },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Update },
            Permission { resource: ResourceType::Reports, action: ActionType::Read },
        ];

        // Transport permissions (supply chain tracking focused)
        let transport_permissions = vec![
            Permission { resource: ResourceType::Product, action: ActionType::Read },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Create },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Read },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Update },
            Permission { resource: ResourceType::Reports, action: ActionType::Read },
        ];

        // Supplier permissions (limited to their products)
        let supplier_permissions = vec![
            Permission { resource: ResourceType::Product, action: ActionType::Create },
            Permission { resource: ResourceType::Product, action: ActionType::Read },
            Permission { resource: ResourceType::Product, action: ActionType::Update },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Create },
            Permission { resource: ResourceType::SupplyChain, action: ActionType::Read },
            Permission { resource: ResourceType::Reports, action: ActionType::Read },
        ];

        // Store permissions for each role
        self.role_permissions.insert(UserRole::Owner, owner_permissions.into_iter().collect());
        self.role_permissions.insert(UserRole::Manager, manager_permissions.into_iter().collect());
        self.role_permissions.insert(UserRole::Warehouse, warehouse_permissions.into_iter().collect());
        self.role_permissions.insert(UserRole::Transport, transport_permissions.into_iter().collect());
        self.role_permissions.insert(UserRole::Supplier, supplier_permissions.into_iter().collect());
    }

    // Set owner role (used during company creation)
    pub fn set_owner_role(&mut self, company_id: String, user_id: String) {
        self.user_roles
            .entry(company_id.clone())
            .or_insert_with(HashMap::new)
            .insert(user_id.clone(), UserRole::Owner);

        // Log the assignment
        self.log_permission_audit(
            user_id,
            company_id,
            ResourceType::User,
            ActionType::Update,
            true,
            "Owner role set".to_string(),
        );
    }

    // Assign role to user in a company
    pub fn assign_role(
        &mut self,
        company_id: String,
        user_id: String,
        role: UserRole,
        assigned_by: String,
    ) -> Result<(), PermissionError> {
        // Check if assigner has permission to assign roles
        if !self.has_permission(&assigned_by, &company_id, &ResourceType::User, &ActionType::Manage) {
            return Err(PermissionError::InsufficientPermissions);
        }

        // Cannot assign owner role (only transfer)
        if role == UserRole::Owner {
            return Err(PermissionError::CannotModifyOwner);
        }

        // Assign the role
        self.user_roles
            .entry(company_id.clone())
            .or_insert_with(HashMap::new)
            .insert(user_id.clone(), role.clone());

        // Log the assignment
        self.log_permission_audit(
            user_id,
            company_id,
            ResourceType::User,
            ActionType::Update,
            true,
            format!("Role assigned: {:?}", role),
        );

        Ok(())
    }

    // Get user's role in a company
    pub fn get_user_role(&self, company_id: &str, user_id: &str) -> Option<&UserRole> {
        self.user_roles
            .get(company_id)?
            .get(user_id)
    }

    // Check if user has a specific permission
    pub fn has_permission(
        &mut self,
        user_id: &str,
        company_id: &str,
        resource: &ResourceType,
        action: &ActionType,
    ) -> bool {
        // Check custom permissions first
        if let Some(company_perms) = self.user_custom_permissions.get(company_id) {
            if let Some(user_perms) = company_perms.get(user_id) {
                let permission = Permission {
                    resource: resource.clone(),
                    action: action.clone(),
                };
                if user_perms.contains(&permission) {
                    self.log_permission_audit(
                        user_id.to_string(),
                        company_id.to_string(),
                        resource.clone(),
                        action.clone(),
                        true,
                        "Custom permission granted".to_string(),
                    );
                    return true;
                }
            }
        }

        // Check role-based permissions
        if let Some(role) = self.get_user_role(company_id, user_id) {
            if let Some(role_perms) = self.role_permissions.get(role) {
                let permission = Permission {
                    resource: resource.clone(),
                    action: action.clone(),
                };

                // Check for exact permission match
                if role_perms.contains(&permission) {
                    self.log_permission_audit(
                        user_id.to_string(),
                        company_id.to_string(),
                        resource.clone(),
                        action.clone(),
                        true,
                        format!("Role permission granted: {:?}", role),
                    );
                    return true;
                }

                // Check for "Manage" action (implies all actions)
                let manage_permission = Permission {
                    resource: resource.clone(),
                    action: ActionType::Manage,
                };
                if role_perms.contains(&manage_permission) {
                    self.log_permission_audit(
                        user_id.to_string(),
                        company_id.to_string(),
                        resource.clone(),
                        action.clone(),
                        true,
                        format!("Manage permission granted: {:?}", role),
                    );
                    return true;
                }
            }
        }

        // Permission denied
        self.log_permission_audit(
            user_id.to_string(),
            company_id.to_string(),
            resource.clone(),
            action.clone(),
            false,
            "Permission denied".to_string(),
        );
        false
    }

    // Grant custom permission to user
    pub fn grant_custom_permission(
        &mut self,
        company_id: String,
        user_id: String,
        resource: ResourceType,
        action: ActionType,
        granted_by: String,
    ) -> Result<(), PermissionError> {
        // Check if granter has permission to manage permissions
        if !self.has_permission(&granted_by, &company_id, &ResourceType::User, &ActionType::Manage) {
            return Err(PermissionError::InsufficientPermissions);
        }

        let permission = Permission { resource, action };

        self.user_custom_permissions
            .entry(company_id.clone())
            .or_insert_with(HashMap::new)
            .entry(user_id.clone())
            .or_insert_with(HashSet::new)
            .insert(permission.clone());

        // Log the permission grant
        self.log_permission_audit(
            user_id,
            company_id,
            permission.resource,
            permission.action,
            true,
            format!("Custom permission granted by {}", granted_by),
        );

        Ok(())
    }

    // Revoke custom permission from user
    pub fn revoke_custom_permission(
        &mut self,
        company_id: String,
        user_id: String,
        resource: ResourceType,
        action: ActionType,
        revoked_by: String,
    ) -> Result<(), PermissionError> {
        // Check if revoker has permission to manage permissions
        if !self.has_permission(&revoked_by, &company_id, &ResourceType::User, &ActionType::Manage) {
            return Err(PermissionError::InsufficientPermissions);
        }

        let permission = Permission { resource, action };

        if let Some(company_perms) = self.user_custom_permissions.get_mut(&company_id) {
            if let Some(user_perms) = company_perms.get_mut(&user_id) {
                user_perms.remove(&permission);

                // Log the permission revocation
                self.log_permission_audit(
                    user_id,
                    company_id,
                    permission.resource,
                    permission.action,
                    false,
                    format!("Custom permission revoked by {}", revoked_by),
                );
            }
        }

        Ok(())
    }

    // Get all permissions for a user in a company
    pub fn get_user_permissions(&self, company_id: &str, user_id: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        // Add role-based permissions
        if let Some(role) = self.get_user_role(company_id, user_id) {
            if let Some(role_perms) = self.role_permissions.get(role) {
                permissions.extend(role_perms.clone());
            }
        }

        // Add custom permissions
        if let Some(company_perms) = self.user_custom_permissions.get(company_id) {
            if let Some(user_perms) = company_perms.get(user_id) {
                permissions.extend(user_perms.clone());
            }
        }

        permissions
    }

    // Get all users with a specific role in a company
    pub fn get_users_with_role(&self, company_id: &str, role: &UserRole) -> Vec<String> {
        if let Some(company_roles) = self.user_roles.get(company_id) {
            company_roles
                .iter()
                .filter(|(_, user_role)| *user_role == role)
                .map(|(user_id, _)| user_id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    // Remove user from company (remove all roles and permissions)
    pub fn remove_user_from_company(&mut self, company_id: &str, user_id: &str) {
        // Remove role assignment
        if let Some(company_roles) = self.user_roles.get_mut(company_id) {
            company_roles.remove(user_id);
        }

        // Remove custom permissions
        if let Some(company_perms) = self.user_custom_permissions.get_mut(company_id) {
            company_perms.remove(user_id);
        }

        // Log the removal
        self.log_permission_audit(
            user_id.to_string(),
            company_id.to_string(),
            ResourceType::User,
            ActionType::Delete,
            true,
            "User removed from company".to_string(),
        );
    }

    // Log permission audit entry
    fn log_permission_audit(
        &mut self,
        user_id: String,
        company_id: String,
        resource: ResourceType,
        action: ActionType,
        granted: bool,
        reason: String,
    ) {
        let entry = PermissionAuditEntry {
            id: format!("audit_{}", self.permission_logs.len() + 1),
            user_id,
            company_id,
            resource,
            action,
            granted,
            reason,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.permission_logs.push(entry);
    }

    // Get permission audit logs for a user
    pub fn get_user_audit_logs(&self, user_id: &str) -> Vec<&PermissionAuditEntry> {
        self.permission_logs
            .iter()
            .filter(|log| log.user_id == user_id)
            .collect()
    }

    // Get permission audit logs for a company
    pub fn get_company_audit_logs(&self, company_id: &str) -> Vec<&PermissionAuditEntry> {
        self.permission_logs
            .iter()
            .filter(|log| log.company_id == company_id)
            .collect()
    }

    // Get permission statistics
    pub fn get_permission_stats(&self) -> PermissionStats {
        let total_role_assignments = self.user_roles
            .values()
            .map(|company_roles| company_roles.len())
            .sum();

        let total_custom_permissions = self.user_custom_permissions
            .values()
            .map(|company_perms| company_perms.values().map(|perms| perms.len()).sum::<usize>())
            .sum();

        let total_audit_entries = self.permission_logs.len();

        let granted_permissions = self.permission_logs
            .iter()
            .filter(|log| log.granted)
            .count();

        PermissionStats {
            total_role_assignments,
            total_custom_permissions,
            total_audit_entries,
            granted_permissions,
            denied_permissions: total_audit_entries - granted_permissions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStats {
    pub total_role_assignments: usize,
    pub total_custom_permissions: usize,
    pub total_audit_entries: usize,
    pub granted_permissions: usize,
    pub denied_permissions: usize,
}

impl Default for RolePermissionSystem {
    fn default() -> Self {
        Self::new()
    }
}