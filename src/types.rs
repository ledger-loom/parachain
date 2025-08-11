use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Owner,
    Manager,
    Warehouse,
    Transport,
    Supplier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub wallet_address: Option<String>,
    pub profile: UserProfile,
    pub verified: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub company_id: Option<String>,
    pub role: Option<UserRole>,
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub members: Vec<CompanyMember>,
    pub settings: CompanySettings,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyMember {
    pub user_id: String,
    pub role: UserRole,
    pub added_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanySettings {
    pub industry: String,
    pub location: String,
    pub preferences: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub company_id: String,
    pub attributes: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProductStatus {
    Created,
    Manufactured,
    InTransit,
    Delivered,
    Returned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainEntry {
    pub id: String,
    pub product_id: String,
    pub status: ProductStatus,
    pub location: Location,
    pub timestamp: u64,
    pub operator_id: String,
    pub notes: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub address: String,
    pub coordinates: Option<Coordinates>,
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductJourney {
    pub product_id: String,
    pub entries: Vec<SupplyChainEntry>,
    pub current_status: ProductStatus,
    pub current_location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub timestamp: u64,
    pub details: HashMap<String, String>,
}

pub type AccountId = String;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type Timestamp = u64;