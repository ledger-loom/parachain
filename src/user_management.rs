use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserManagement {
    users: HashMap<String, User>,
    email_to_user_id: HashMap<String, String>,
    wallet_to_user_id: HashMap<String, String>,
    verification_requests: HashMap<String, VerificationRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub user_id: String,
    pub verification_type: VerificationType,
    pub documents: Vec<Document>,
    pub status: VerificationStatus,
    pub submitted_at: u64,
    pub reviewed_at: Option<u64>,
    pub reviewer_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationType {
    Identity,
    Business,
    Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
    RequiresMoreInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub document_type: DocumentType,
    pub hash: String, // Hash of document for integrity
    pub uploaded_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    GovernmentId,
    BusinessLicense,
    ProofOfAddress,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserManagementError {
    UserAlreadyExists,
    UserNotFound,
    InvalidEmail,
    InvalidWalletAddress,
    EmailAlreadyTaken,
    WalletAlreadyLinked,
    VerificationPending,
    InvalidCredentials,
    InsufficientPermissions,
}

impl UserManagement {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            email_to_user_id: HashMap::new(),
            wallet_to_user_id: HashMap::new(),
            verification_requests: HashMap::new(),
        }
    }

    // User Registration
    pub fn register_user(
        &mut self,
        email: String,
        password_hash: String,
        name: String,
        wallet_address: Option<String>,
    ) -> Result<String, UserManagementError> {
        // Validate email format (basic validation)
        if !email.contains('@') {
            return Err(UserManagementError::InvalidEmail);
        }

        // Check if email is already taken
        if self.email_to_user_id.contains_key(&email) {
            return Err(UserManagementError::EmailAlreadyTaken);
        }

        // Check if wallet address is already linked
        if let Some(ref wallet) = wallet_address {
            if self.wallet_to_user_id.contains_key(wallet) {
                return Err(UserManagementError::WalletAlreadyLinked);
            }
        }

        // Generate user ID
        let user_id = format!("user_{}", self.users.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create user
        let user = User {
            id: user_id.clone(),
            email: email.clone(),
            wallet_address: wallet_address.clone(),
            profile: UserProfile {
                name,
                company_id: None,
                role: None,
                settings: HashMap::new(),
            },
            verified: false, // Start unverified
            created_at: current_time,
        };

        // Store user
        self.users.insert(user_id.clone(), user);
        self.email_to_user_id.insert(email, user_id.clone());
        
        if let Some(wallet) = wallet_address {
            self.wallet_to_user_id.insert(wallet, user_id.clone());
        }

        Ok(user_id)
    }

    // User Authentication
    pub fn authenticate_user(&self, email: &str, password_hash: &str) -> Result<&User, UserManagementError> {
        let user_id = self.email_to_user_id.get(email)
            .ok_or(UserManagementError::UserNotFound)?;
        
        let user = self.users.get(user_id)
            .ok_or(UserManagementError::UserNotFound)?;

        // In a real implementation, you would verify the password hash
        // For demo purposes, we'll assume authentication succeeds if user exists
        Ok(user)
    }

    // Link wallet to existing user
    pub fn link_wallet(&mut self, user_id: &str, wallet_address: String) -> Result<(), UserManagementError> {
        // Check if wallet is already linked
        if self.wallet_to_user_id.contains_key(&wallet_address) {
            return Err(UserManagementError::WalletAlreadyLinked);
        }

        // Get user and update
        let user = self.users.get_mut(user_id)
            .ok_or(UserManagementError::UserNotFound)?;

        user.wallet_address = Some(wallet_address.clone());
        self.wallet_to_user_id.insert(wallet_address, user_id.to_string());

        Ok(())
    }

    // Submit verification request
    pub fn submit_verification(
        &mut self,
        user_id: &str,
        verification_type: VerificationType,
        documents: Vec<Document>,
    ) -> Result<String, UserManagementError> {
        // Check if user exists
        if !self.users.contains_key(user_id) {
            return Err(UserManagementError::UserNotFound);
        }

        let request_id = format!("verify_{}", self.verification_requests.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let verification_request = VerificationRequest {
            user_id: user_id.to_string(),
            verification_type,
            documents,
            status: VerificationStatus::Pending,
            submitted_at: current_time,
            reviewed_at: None,
            reviewer_notes: None,
        };

        self.verification_requests.insert(request_id.clone(), verification_request);
        Ok(request_id)
    }

    // Review verification request (admin function)
    pub fn review_verification(
        &mut self,
        request_id: &str,
        status: VerificationStatus,
        reviewer_notes: Option<String>,
    ) -> Result<(), UserManagementError> {
        let request = self.verification_requests.get_mut(request_id)
            .ok_or(UserManagementError::UserNotFound)?;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        request.status = status.clone();
        request.reviewed_at = Some(current_time);
        request.reviewer_notes = reviewer_notes;

        // If approved, mark user as verified
        if matches!(status, VerificationStatus::Approved) {
            if let Some(user) = self.users.get_mut(&request.user_id) {
                user.verified = true;
            }
        }

        Ok(())
    }

    // Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }

    // Get user by email
    pub fn get_user_by_email(&self, email: &str) -> Option<&User> {
        let user_id = self.email_to_user_id.get(email)?;
        self.users.get(user_id)
    }

    // Get user by wallet address
    pub fn get_user_by_wallet(&self, wallet_address: &str) -> Option<&User> {
        let user_id = self.wallet_to_user_id.get(wallet_address)?;
        self.users.get(user_id)
    }

    // Update user profile
    pub fn update_user_profile(
        &mut self,
        user_id: &str,
        name: Option<String>,
        settings: Option<HashMap<String, String>>,
    ) -> Result<(), UserManagementError> {
        let user = self.users.get_mut(user_id)
            .ok_or(UserManagementError::UserNotFound)?;

        if let Some(name) = name {
            user.profile.name = name;
        }

        if let Some(settings) = settings {
            user.profile.settings.extend(settings);
        }

        Ok(())
    }

    // Get verification status
    pub fn get_verification_status(&self, user_id: &str) -> Option<bool> {
        self.users.get(user_id).map(|user| user.verified)
    }

    // List pending verifications (admin function)
    pub fn get_pending_verifications(&self) -> Vec<&VerificationRequest> {
        self.verification_requests
            .values()
            .filter(|request| matches!(request.status, VerificationStatus::Pending))
            .collect()
    }

    // Get user statistics
    pub fn get_user_stats(&self) -> UserStats {
        let total_users = self.users.len();
        let verified_users = self.users.values().filter(|user| user.verified).count();
        let pending_verifications = self.verification_requests.values()
            .filter(|request| matches!(request.status, VerificationStatus::Pending))
            .count();

        UserStats {
            total_users,
            verified_users,
            unverified_users: total_users - verified_users,
            pending_verifications,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_users: usize,
    pub verified_users: usize,
    pub unverified_users: usize,
    pub pending_verifications: usize,
}

impl Default for UserManagement {
    fn default() -> Self {
        Self::new()
    }
}