use crate::types::*;
use crate::role_permissions::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeSystem {
    qr_codes: HashMap<String, QrCodeData>,
    verification_log: HashMap<String, Vec<QrVerification>>,
    mobile_sessions: HashMap<String, MobileSession>,
    role_system: RolePermissionSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeData {
    pub id: String,
    pub qr_type: QrCodeType,
    pub entity_id: String,
    pub entity_type: EntityType,
    pub company_id: String,
    pub created_by: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub verification_url: String,
    pub blockchain_hash: Option<String>,
    pub metadata: HashMap<String, String>,
    pub access_level: QrAccessLevel,
    pub scan_count: u32,
    pub last_scanned: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QrCodeType {
    Product,
    ProductBatch,
    TrackingEntry,
    Shipment,
    Location,
    Document,
    WarehouseItem,
    QualityCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Product,
    Batch,
    TrackingEntry,
    Shipment,
    Location,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QrAccessLevel {
    Public,     // Anyone can scan and view basic info
    Company,    // Only company members can access full details
    Internal,   // Only specific roles can access
    Private,    // Only creator and admins can access
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrVerification {
    pub id: String,
    pub qr_code_id: String,
    pub scanned_by: Option<String>,
    pub scanned_at: u64,
    pub scanner_location: Option<GpsLocation>,
    pub device_info: Option<DeviceInfo>,
    pub verification_result: VerificationResult,
    pub access_granted: bool,
    pub action_taken: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f32>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    Valid,
    Expired,
    Invalid,
    AccessDenied,
    NotFound,
    Tampered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub device_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub permissions: Vec<MobilePermission>,
    pub offline_data: HashMap<String, String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MobilePermission {
    ScanQrCodes,
    UpdateStatus,
    ViewDetails,
    CreateEntries,
    OfflineAccess,
    GpsAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Online,
    Offline,
    Syncing,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeRequest {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub company_id: String,
    pub access_level: QrAccessLevel,
    pub expires_at: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrScanRequest {
    pub qr_code_id: String,
    pub scanner_user_id: Option<String>,
    pub device_info: Option<DeviceInfo>,
    pub location: Option<GpsLocation>,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrScanResponse {
    pub verification_result: VerificationResult,
    pub access_granted: bool,
    pub entity_data: Option<QrEntityData>,
    pub available_actions: Vec<QrAction>,
    pub verification_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEntityData {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub status: String,
    pub location: Option<String>,
    pub company_name: String,
    pub created_at: u64,
    pub attributes: HashMap<String, String>,
    pub tracking_history: Option<Vec<TrackingSnapshot>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSnapshot {
    pub timestamp: u64,
    pub status: String,
    pub location: String,
    pub operator: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrAction {
    pub action_id: String,
    pub display_name: String,
    pub description: String,
    pub requires_auth: bool,
    pub required_role: Option<UserRole>,
    pub parameters: Vec<ActionParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParameter {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QrCodeSystemError {
    QrCodeNotFound,
    EntityNotFound,
    AccessDenied,
    InvalidQrCode,
    ExpiredQrCode,
    InvalidRequest,
    GenerationFailed,
    VerificationFailed,
    SessionExpired,
    InsufficientPermissions,
}

impl QrCodeSystem {
    pub fn new() -> Self {
        Self {
            qr_codes: HashMap::new(),
            verification_log: HashMap::new(),
            mobile_sessions: HashMap::new(),
            role_system: RolePermissionSystem::new(),
        }
    }

    // Role management methods
    pub fn set_owner_role(&mut self, company_id: String, user_id: String) {
        self.role_system.set_owner_role(company_id, user_id);
    }

    pub fn assign_role(
        &mut self,
        company_id: String,
        user_id: String,
        role: UserRole,
        assigned_by: String,
    ) -> Result<(), PermissionError> {
        self.role_system.assign_role(company_id, user_id, role, assigned_by)
    }

    // Generate QR code for an entity
    pub fn generate_qr_code(
        &mut self,
        user_id: String,
        company_id: String,
        request: QrCodeRequest,
    ) -> Result<String, QrCodeSystemError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::Product, &ActionType::Create) {
            return Err(QrCodeSystemError::InsufficientPermissions);
        }

        // Generate QR code ID
        let qr_id = format!("qr_{}_{}", 
            format!("{:?}", request.entity_type).to_lowercase(),
            self.qr_codes.len() + 1
        );

        // Create verification URL
        let verification_url = format!(
            "https://supplychainmanager.io/verify/{}", 
            qr_id
        );

        // Generate blockchain hash (mock implementation)
        let blockchain_hash = self.generate_blockchain_hash(&request.entity_id, &company_id);

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create QR code data
        let qr_code = QrCodeData {
            id: qr_id.clone(),
            qr_type: self.entity_type_to_qr_type(&request.entity_type),
            entity_id: request.entity_id,
            entity_type: request.entity_type,
            company_id: company_id.clone(),
            created_by: user_id,
            created_at: current_time,
            expires_at: request.expires_at,
            verification_url,
            blockchain_hash: Some(blockchain_hash),
            metadata: request.metadata,
            access_level: request.access_level,
            scan_count: 0,
            last_scanned: None,
        };

        // Store QR code
        self.qr_codes.insert(qr_id.clone(), qr_code);

        Ok(qr_id)
    }

    // Verify QR code when scanned
    pub fn verify_qr_code(
        &mut self,
        request: QrScanRequest,
    ) -> Result<QrScanResponse, QrCodeSystemError> {
        // Find QR code
        let qr_code = self.qr_codes.get_mut(&request.qr_code_id)
            .ok_or(QrCodeSystemError::QrCodeNotFound)?;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if expired
        if let Some(expires_at) = qr_code.expires_at {
            if current_time > expires_at {
                return Ok(QrScanResponse {
                    verification_result: VerificationResult::Expired,
                    access_granted: false,
                    entity_data: None,
                    available_actions: vec![],
                    verification_id: self.generate_verification_id(),
                });
            }
        }

        // Check access permissions
        let access_granted = self.check_qr_access(qr_code, &request);

        // Update scan statistics
        qr_code.scan_count += 1;
        qr_code.last_scanned = Some(current_time);

        // Generate verification record
        let verification_id = self.generate_verification_id();
        let verification = QrVerification {
            id: verification_id.clone(),
            qr_code_id: request.qr_code_id.clone(),
            scanned_by: request.scanner_user_id.clone(),
            scanned_at: current_time,
            scanner_location: request.location,
            device_info: request.device_info,
            verification_result: if access_granted { 
                VerificationResult::Valid 
            } else { 
                VerificationResult::AccessDenied 
            },
            access_granted,
            action_taken: request.action,
        };

        // Store verification
        self.verification_log
            .entry(request.qr_code_id)
            .or_insert_with(Vec::new)
            .push(verification);

        // Prepare response
        let entity_data = if access_granted {
            Some(self.get_entity_data(&qr_code.entity_type, &qr_code.entity_id)?)
        } else {
            None
        };

        let available_actions = if access_granted {
            self.get_available_actions(&qr_code.qr_type, &request.scanner_user_id)
        } else {
            vec![]
        };

        Ok(QrScanResponse {
            verification_result: VerificationResult::Valid,
            access_granted,
            entity_data,
            available_actions,
            verification_id,
        })
    }

    // Create mobile session for offline access
    pub fn create_mobile_session(
        &mut self,
        user_id: String,
        company_id: String,
        device_id: String,
    ) -> Result<String, QrCodeSystemError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Read) {
            return Err(QrCodeSystemError::InsufficientPermissions);
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session_id = format!("session_{}", self.mobile_sessions.len() + 1);

        // Determine permissions based on user role
        let user_role = self.role_system.get_user_role(&user_id, &company_id);
        let permissions = self.get_mobile_permissions(&user_role);

        let session = MobileSession {
            session_id: session_id.clone(),
            user_id: Some(user_id),
            device_id,
            created_at: current_time,
            expires_at: current_time + 24 * 60 * 60, // 24 hours
            permissions,
            offline_data: HashMap::new(),
            sync_status: SyncStatus::Online,
        };

        self.mobile_sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    // Get QR code statistics
    pub fn get_qr_statistics(&self, company_id: &str) -> QrStatistics {
        let company_qr_codes: Vec<_> = self.qr_codes.values()
            .filter(|qr| qr.company_id == company_id)
            .collect();

        let total_qr_codes = company_qr_codes.len();
        let total_scans = company_qr_codes.iter().map(|qr| qr.scan_count).sum::<u32>();
        
        let active_qr_codes = company_qr_codes.iter()
            .filter(|qr| qr.expires_at.is_none() || qr.expires_at.unwrap() > 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs())
            .count();

        let qr_types_breakdown = self.calculate_qr_type_breakdown(&company_qr_codes);

        QrStatistics {
            total_qr_codes,
            active_qr_codes,
            expired_qr_codes: total_qr_codes - active_qr_codes,
            total_scans,
            unique_scanners: self.count_unique_scanners(company_id),
            qr_types_breakdown,
        }
    }

    // Helper methods
    fn generate_blockchain_hash(&self, entity_id: &str, company_id: &str) -> String {
        // Mock blockchain hash generation
        format!("0x{:x}", 
            std::collections::hash_map::DefaultHasher::new()
                .chain_update(entity_id.as_bytes())
                .chain_update(company_id.as_bytes())
                .finish()
        )
    }

    fn entity_type_to_qr_type(&self, entity_type: &EntityType) -> QrCodeType {
        match entity_type {
            EntityType::Product => QrCodeType::Product,
            EntityType::Batch => QrCodeType::ProductBatch,
            EntityType::TrackingEntry => QrCodeType::TrackingEntry,
            EntityType::Shipment => QrCodeType::Shipment,
            EntityType::Location => QrCodeType::Location,
            EntityType::Document => QrCodeType::Document,
        }
    }

    fn check_qr_access(&self, qr_code: &QrCodeData, request: &QrScanRequest) -> bool {
        match qr_code.access_level {
            QrAccessLevel::Public => true,
            QrAccessLevel::Company => {
                // Check if scanner belongs to the same company
                if let Some(ref user_id) = request.scanner_user_id {
                    self.role_system.has_permission(user_id, &qr_code.company_id, &ResourceType::Product, &ActionType::Read)
                } else {
                    false
                }
            },
            QrAccessLevel::Internal | QrAccessLevel::Private => {
                if let Some(ref user_id) = request.scanner_user_id {
                    let user_role = self.role_system.get_user_role(user_id, &qr_code.company_id);
                    matches!(user_role, Some(UserRole::Manager) | Some(UserRole::Owner))
                } else {
                    false
                }
            },
        }
    }

    fn get_entity_data(&self, entity_type: &EntityType, entity_id: &str) -> Result<QrEntityData, QrCodeSystemError> {
        // Mock implementation - in production, this would query the actual entity
        Ok(QrEntityData {
            entity_type: entity_type.clone(),
            entity_id: entity_id.to_string(),
            display_name: format!("Entity {}", entity_id),
            description: Some("Mock entity description".to_string()),
            status: "Active".to_string(),
            location: Some("Warehouse A".to_string()),
            company_name: "Supply Chain Solutions Inc".to_string(),
            created_at: 1640995200,
            attributes: HashMap::new(),
            tracking_history: Some(vec![
                TrackingSnapshot {
                    timestamp: 1640995200,
                    status: "Created".to_string(),
                    location: "Factory".to_string(),
                    operator: "System".to_string(),
                    notes: None,
                },
                TrackingSnapshot {
                    timestamp: 1640995800,
                    status: "In Transit".to_string(),
                    location: "Warehouse A".to_string(),
                    operator: "Transport Team".to_string(),
                    notes: Some("Shipped via truck".to_string()),
                },
            ]),
        })
    }

    fn get_available_actions(&self, qr_type: &QrCodeType, user_id: &Option<String>) -> Vec<QrAction> {
        let mut actions = vec![];

        match qr_type {
            QrCodeType::Product => {
                actions.push(QrAction {
                    action_id: "view_details".to_string(),
                    display_name: "View Product Details".to_string(),
                    description: "View complete product information".to_string(),
                    requires_auth: false,
                    required_role: None,
                    parameters: vec![],
                });

                if user_id.is_some() {
                    actions.push(QrAction {
                        action_id: "update_status".to_string(),
                        display_name: "Update Status".to_string(),
                        description: "Update product status".to_string(),
                        requires_auth: true,
                        required_role: Some(UserRole::Warehouse),
                        parameters: vec![
                            ActionParameter {
                                name: "new_status".to_string(),
                                parameter_type: "enum".to_string(),
                                required: true,
                                default_value: None,
                                validation_rules: vec!["valid_status".to_string()],
                            },
                            ActionParameter {
                                name: "notes".to_string(),
                                parameter_type: "text".to_string(),
                                required: false,
                                default_value: None,
                                validation_rules: vec!["max_length:500".to_string()],
                            },
                        ],
                    });
                }
            },
            QrCodeType::Location => {
                actions.push(QrAction {
                    action_id: "check_in".to_string(),
                    display_name: "Check In".to_string(),
                    description: "Register arrival at this location".to_string(),
                    requires_auth: true,
                    required_role: Some(UserRole::Transport),
                    parameters: vec![],
                });
            },
            _ => {
                // Default actions for other QR types
                actions.push(QrAction {
                    action_id: "view_details".to_string(),
                    display_name: "View Details".to_string(),
                    description: "View information".to_string(),
                    requires_auth: false,
                    required_role: None,
                    parameters: vec![],
                });
            }
        }

        actions
    }

    fn get_mobile_permissions(&self, user_role: &Option<UserRole>) -> Vec<MobilePermission> {
        let mut permissions = vec![MobilePermission::ScanQrCodes, MobilePermission::ViewDetails];

        if let Some(role) = user_role {
            match role {
                UserRole::Owner | UserRole::Manager => {
                    permissions.extend(vec![
                        MobilePermission::UpdateStatus,
                        MobilePermission::CreateEntries,
                        MobilePermission::OfflineAccess,
                        MobilePermission::GpsAccess,
                    ]);
                },
                UserRole::Warehouse | UserRole::Transport => {
                    permissions.extend(vec![
                        MobilePermission::UpdateStatus,
                        MobilePermission::GpsAccess,
                    ]);
                },
                _ => {}
            }
        }

        permissions
    }

    fn generate_verification_id(&self) -> String {
        format!("verify_{}", self.verification_log.len() + 1)
    }

    fn calculate_qr_type_breakdown(&self, qr_codes: &[&QrCodeData]) -> HashMap<QrCodeType, usize> {
        let mut breakdown = HashMap::new();
        for qr_code in qr_codes {
            *breakdown.entry(qr_code.qr_type.clone()).or_insert(0) += 1;
        }
        breakdown
    }

    fn count_unique_scanners(&self, company_id: &str) -> usize {
        let mut scanners = std::collections::HashSet::new();
        
        for qr_code in self.qr_codes.values() {
            if qr_code.company_id == company_id {
                if let Some(verifications) = self.verification_log.get(&qr_code.id) {
                    for verification in verifications {
                        if let Some(ref scanned_by) = verification.scanned_by {
                            scanners.insert(scanned_by.clone());
                        }
                    }
                }
            }
        }

        scanners.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrStatistics {
    pub total_qr_codes: usize,
    pub active_qr_codes: usize,
    pub expired_qr_codes: usize,
    pub total_scans: u32,
    pub unique_scanners: usize,
    pub qr_types_breakdown: HashMap<QrCodeType, usize>,
}

// Additional trait for hash computation
trait HashChain {
    fn chain_update(self, data: &[u8]) -> Self;
    fn finish(self) -> u64;
}

impl HashChain for std::collections::hash_map::DefaultHasher {
    fn chain_update(mut self, data: &[u8]) -> Self {
        use std::hash::{Hash, Hasher};
        data.hash(&mut self);
        self
    }

    fn finish(self) -> u64 {
        use std::hash::Hasher;
        self.finish()
    }
}

impl Default for QrCodeSystem {
    fn default() -> Self {
        Self::new()
    }
}