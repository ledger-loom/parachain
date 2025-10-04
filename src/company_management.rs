use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyManagement {
    companies: HashMap<String, Company>,
    user_companies: HashMap<String, Vec<String>>, // user_id -> company_ids
    company_invitations: HashMap<String, CompanyInvitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInvitation {
    pub id: String,
    pub company_id: String,
    pub inviter_user_id: String,
    pub invitee_email: String,
    pub invitee_user_id: Option<String>, // Set when user accepts
    pub role: UserRole,
    pub status: InvitationStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipTransfer {
    pub id: String,
    pub company_id: String,
    pub current_owner_id: String,
    pub new_owner_id: String,
    pub status: TransferStatus,
    pub requested_at: u64,
    pub completed_at: Option<u64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    Accepted,
    Declined,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompanyManagementError {
    CompanyNotFound,
    UserNotFound,
    UserNotMember,
    UserNotOwner,
    InsufficientPermissions,
    CompanyAlreadyExists,
    UserAlreadyMember,
    InvitationNotFound,
    InvitationExpired,
    InvitationAlreadyProcessed,
    CannotTransferToSelf,
    TransferNotFound,
    InvalidCompanyName,
    MaxMembersReached,
}

impl CompanyManagement {
    pub fn new() -> Self {
        Self {
            companies: HashMap::new(),
            user_companies: HashMap::new(),
            company_invitations: HashMap::new(),
        }
    }

    // Create a new company
    pub fn create_company(
        &mut self,
        owner_user_id: String,
        name: String,
        description: String,
        industry: String,
        location: String,
    ) -> Result<String, CompanyManagementError> {
        // Validate company name
        if name.trim().is_empty() {
            return Err(CompanyManagementError::InvalidCompanyName);
        }

        // Generate company ID
        let company_id = format!("company_{}", self.companies.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create company
        let company = Company {
            id: company_id.clone(),
            name,
            description,
            owner_id: owner_user_id.clone(),
            members: vec![CompanyMember {
                user_id: owner_user_id.clone(),
                role: UserRole::Owner,
                added_at: current_time,
            }],
            settings: CompanySettings {
                industry,
                location,
                preferences: HashMap::new(),
            },
            created_at: current_time,
        };

        // Store company
        self.companies.insert(company_id.clone(), company);
        
        // Update user-company mapping
        self.user_companies
            .entry(owner_user_id)
            .or_insert_with(Vec::new)
            .push(company_id.clone());

        Ok(company_id)
    }

    // Invite a user to join the company
    pub fn invite_user(
        &mut self,
        company_id: String,
        inviter_user_id: String,
        invitee_email: String,
        role: UserRole,
        message: Option<String>,
    ) -> Result<String, CompanyManagementError> {
        // Check if company exists and inviter has permissions
        let company = self.companies.get(&company_id)
            .ok_or(CompanyManagementError::CompanyNotFound)?;

        // Check if inviter is owner or manager
        let inviter_member = company.members.iter()
            .find(|m| m.user_id == inviter_user_id)
            .ok_or(CompanyManagementError::UserNotMember)?;

        if !matches!(inviter_member.role, UserRole::Owner | UserRole::Manager) {
            return Err(CompanyManagementError::InsufficientPermissions);
        }

        // Check if user is already a member
        let is_already_member = company.members.iter()
            .any(|m| {
                // This would need to be checked against email in a real system
                false // For now, assume not a member
            });

        if is_already_member {
            return Err(CompanyManagementError::UserAlreadyMember);
        }

        // Create invitation
        let invitation_id = format!("invite_{}", self.company_invitations.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let invitation = CompanyInvitation {
            id: invitation_id.clone(),
            company_id,
            inviter_user_id,
            invitee_email,
            invitee_user_id: None,
            role,
            status: InvitationStatus::Pending,
            created_at: current_time,
            expires_at: current_time + (7 * 24 * 60 * 60), // Expires in 7 days
            message,
        };

        self.company_invitations.insert(invitation_id.clone(), invitation);
        Ok(invitation_id)
    }

    // Accept company invitation
    pub fn accept_invitation(
        &mut self,
        invitation_id: String,
        user_id: String,
    ) -> Result<(), CompanyManagementError> {
        // Get invitation
        let invitation = self.company_invitations.get_mut(&invitation_id)
            .ok_or(CompanyManagementError::InvitationNotFound)?;

        // Check if invitation is still pending
        if invitation.status != InvitationStatus::Pending {
            return Err(CompanyManagementError::InvitationAlreadyProcessed);
        }

        // Check if invitation has expired
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time > invitation.expires_at {
            invitation.status = InvitationStatus::Expired;
            return Err(CompanyManagementError::InvitationExpired);
        }

        // Get company
        let company = self.companies.get_mut(&invitation.company_id)
            .ok_or(CompanyManagementError::CompanyNotFound)?;

        // Add user to company
        let new_member = CompanyMember {
            user_id: user_id.clone(),
            role: invitation.role.clone(),
            added_at: current_time,
        };

        company.members.push(new_member);

        // Update user-company mapping
        self.user_companies
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(invitation.company_id.clone());

        // Update invitation status
        invitation.status = InvitationStatus::Accepted;
        invitation.invitee_user_id = Some(user_id);

        Ok(())
    }

    // Decline company invitation
    pub fn decline_invitation(&mut self, invitation_id: String) -> Result<(), CompanyManagementError> {
        let invitation = self.company_invitations.get_mut(&invitation_id)
            .ok_or(CompanyManagementError::InvitationNotFound)?;

        if invitation.status != InvitationStatus::Pending {
            return Err(CompanyManagementError::InvitationAlreadyProcessed);
        }

        invitation.status = InvitationStatus::Declined;
        Ok(())
    }

    // Remove user from company
    pub fn remove_member(
        &mut self,
        company_id: String,
        remover_user_id: String,
        target_user_id: String,
    ) -> Result<(), CompanyManagementError> {
        // Get company
        let company = self.companies.get_mut(&company_id)
            .ok_or(CompanyManagementError::CompanyNotFound)?;

        // Check if remover has permissions (owner or manager)
        let remover_member = company.members.iter()
            .find(|m| m.user_id == remover_user_id)
            .ok_or(CompanyManagementError::UserNotMember)?;

        if !matches!(remover_member.role, UserRole::Owner | UserRole::Manager) {
            return Err(CompanyManagementError::InsufficientPermissions);
        }

        // Cannot remove the owner
        if company.owner_id == target_user_id {
            return Err(CompanyManagementError::InsufficientPermissions);
        }

        // Remove member from company
        company.members.retain(|m| m.user_id != target_user_id);

        // Update user-company mapping
        if let Some(user_companies) = self.user_companies.get_mut(&target_user_id) {
            user_companies.retain(|c_id| c_id != &company_id);
        }

        Ok(())
    }

    // Update company settings
    pub fn update_company_settings(
        &mut self,
        company_id: String,
        user_id: String,
        name: Option<String>,
        description: Option<String>,
        industry: Option<String>,
        location: Option<String>,
        preferences: Option<HashMap<String, String>>,
    ) -> Result<(), CompanyManagementError> {
        // Get company
        let company = self.companies.get_mut(&company_id)
            .ok_or(CompanyManagementError::CompanyNotFound)?;

        // Check if user has permissions (owner or manager)
        let member = company.members.iter()
            .find(|m| m.user_id == user_id)
            .ok_or(CompanyManagementError::UserNotMember)?;

        if !matches!(member.role, UserRole::Owner | UserRole::Manager) {
            return Err(CompanyManagementError::InsufficientPermissions);
        }

        // Update settings
        if let Some(name) = name {
            if name.trim().is_empty() {
                return Err(CompanyManagementError::InvalidCompanyName);
            }
            company.name = name;
        }

        if let Some(description) = description {
            company.description = description;
        }

        if let Some(industry) = industry {
            company.settings.industry = industry;
        }

        if let Some(location) = location {
            company.settings.location = location;
        }

        if let Some(preferences) = preferences {
            company.settings.preferences.extend(preferences);
        }

        Ok(())
    }

    // Transfer company ownership
    pub fn request_ownership_transfer(
        &mut self,
        company_id: String,
        current_owner_id: String,
        new_owner_id: String,
        reason: Option<String>,
    ) -> Result<String, CompanyManagementError> {
        // Get company
        let company = self.companies.get(&company_id)
            .ok_or(CompanyManagementError::CompanyNotFound)?;

        // Check if requester is current owner
        if company.owner_id != current_owner_id {
            return Err(CompanyManagementError::UserNotOwner);
        }

        // Check if new owner is different
        if current_owner_id == new_owner_id {
            return Err(CompanyManagementError::CannotTransferToSelf);
        }

        // Check if new owner is a member
        let is_member = company.members.iter()
            .any(|m| m.user_id == new_owner_id);

        if !is_member {
            return Err(CompanyManagementError::UserNotMember);
        }

        // Create transfer request
        let transfer_id = format!("transfer_{}", self.companies.len());
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // For simplicity, we'll complete the transfer immediately
        // In a real system, this might require approval from the new owner
        self.complete_ownership_transfer(company_id, new_owner_id)?;

        Ok(transfer_id)
    }

    // Complete ownership transfer
    fn complete_ownership_transfer(
        &mut self,
        company_id: String,
        new_owner_id: String,
    ) -> Result<(), CompanyManagementError> {
        let company = self.companies.get_mut(&company_id)
            .ok_or(CompanyManagementError::CompanyNotFound)?;

        let old_owner_id = company.owner_id.clone();
        company.owner_id = new_owner_id.clone();

        // Update member roles
        for member in &mut company.members {
            if member.user_id == old_owner_id {
                member.role = UserRole::Manager; // Demote old owner to manager
            } else if member.user_id == new_owner_id {
                member.role = UserRole::Owner; // Promote new owner
            }
        }

        Ok(())
    }

    // Get company by ID
    pub fn get_company(&self, company_id: &str) -> Option<&Company> {
        self.companies.get(company_id)
    }

    // Get companies for a user
    pub fn get_user_companies(&self, user_id: &str) -> Vec<&Company> {
        if let Some(company_ids) = self.user_companies.get(user_id) {
            company_ids.iter()
                .filter_map(|id| self.companies.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    // Get company members
    pub fn get_company_members(&self, company_id: &str) -> Option<&Vec<CompanyMember>> {
        self.companies.get(company_id).map(|company| &company.members)
    }

    // Get pending invitations for a company
    pub fn get_company_invitations(&self, company_id: &str) -> Vec<&CompanyInvitation> {
        self.company_invitations.values()
            .filter(|inv| inv.company_id == company_id && inv.status == InvitationStatus::Pending)
            .collect()
    }

    // Get invitations for an email
    pub fn get_user_invitations(&self, email: &str) -> Vec<&CompanyInvitation> {
        self.company_invitations.values()
            .filter(|inv| inv.invitee_email == email && inv.status == InvitationStatus::Pending)
            .collect()
    }

    // Get company statistics
    pub fn get_company_stats(&self) -> CompanyStats {
        let total_companies = self.companies.len();
        let total_members = self.companies.values()
            .map(|c| c.members.len())
            .sum();
        let pending_invitations = self.company_invitations.values()
            .filter(|inv| inv.status == InvitationStatus::Pending)
            .count();

        CompanyStats {
            total_companies,
            total_members,
            pending_invitations,
            average_members_per_company: if total_companies > 0 {
                total_members as f64 / total_companies as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyStats {
    pub total_companies: usize,
    pub total_members: usize,
    pub pending_invitations: usize,
    pub average_members_per_company: f64,
}

impl Default for CompanyManagement {
    fn default() -> Self {
        Self::new()
    }
}