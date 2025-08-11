use crate::types::*;
use crate::role_permissions::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainTracking {
    tracking_entries: HashMap<String, TrackingEntry>,
    product_journeys: HashMap<String, ProductJourney>,
    location_registry: HashMap<String, LocationInfo>,
    status_rules: HashMap<String, Vec<StatusTransitionRule>>,
    notifications: HashMap<String, Vec<TrackingNotification>>,
    shipments: HashMap<String, Shipment>,
    role_system: RolePermissionSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEntry {
    pub id: String,
    pub product_id: String,
    pub shipment_id: Option<String>,
    pub status: TrackingStatus,
    pub location: LocationInfo,
    pub operator_id: String,
    pub company_id: String,
    pub timestamp: u64,
    pub notes: Option<String>,
    pub metadata: HashMap<String, String>,
    pub documents: Vec<TrackingDocument>,
    pub environmental_data: Option<EnvironmentalData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductJourney {
    pub product_id: String,
    pub tracking_entries: Vec<String>, // Entry IDs in chronological order
    pub current_status: TrackingStatus,
    pub current_location: LocationInfo,
    pub started_at: u64,
    pub estimated_delivery: Option<u64>,
    pub actual_delivery: Option<u64>,
    pub total_distance: Option<f64>,
    pub companies_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub coordinates: Option<Coordinates>,
    pub location_type: LocationType,
    pub company_id: Option<String>,
    pub timezone: String,
    pub contact_info: Option<ContactInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LocationType {
    Warehouse,
    Factory,
    DistributionCenter,
    RetailStore,
    TransportHub,
    Port,
    Airport,
    CustomerLocation,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TrackingStatus {
    Created,
    InProduction,
    QualityCheck,
    ReadyToShip,
    InTransit,
    AtWarehouse,
    OutForDelivery,
    Delivered,
    Returned,
    Lost,
    Damaged,
    Recalled,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransitionRule {
    pub from_status: TrackingStatus,
    pub to_status: TrackingStatus,
    pub required_role: Option<UserRole>,
    pub required_location_type: Option<LocationType>,
    pub automatic: bool, // If true, transition happens automatically under conditions
    pub conditions: Vec<TransitionCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    TimeElapsed(u64), // Seconds since last status
    LocationReached(String), // Location ID
    DocumentUploaded(String), // Document type
    EnvironmentalThreshold { sensor: String, min: Option<f64>, max: Option<f64> },
    UserConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingDocument {
    pub id: String,
    pub document_type: DocumentType,
    pub name: String,
    pub hash: String,
    pub uploaded_by: String,
    pub uploaded_at: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Certificate,
    Photo,
    Invoice,
    Receipt,
    QualityReport,
    CustomsDeclaration,
    InsuranceDocument,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalData {
    pub temperature: Option<f64>, // Celsius
    pub humidity: Option<f64>,    // Percentage
    pub pressure: Option<f64>,    // hPa
    pub vibration: Option<f64>,   // g-force
    pub light_exposure: Option<f64>, // lux
    pub recorded_at: u64,
    pub sensor_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub id: String,
    pub product_ids: Vec<String>,
    pub origin: LocationInfo,
    pub destination: LocationInfo,
    pub carrier_company_id: String,
    pub driver_id: Option<String>,
    pub vehicle_id: Option<String>,
    pub status: ShipmentStatus,
    pub created_at: u64,
    pub estimated_delivery: Option<u64>,
    pub actual_delivery: Option<u64>,
    pub tracking_number: Option<String>,
    pub route: Vec<String>, // Location IDs
    pub special_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShipmentStatus {
    Pending,
    PickedUp,
    InTransit,
    Delivered,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingNotification {
    pub id: String,
    pub notification_type: NotificationType,
    pub product_id: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub created_at: u64,
    pub recipients: Vec<String>, // User IDs
    pub acknowledged_by: Vec<String>, // User IDs who acknowledged
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    StatusChange,
    LocationUpdate,
    DelayAlert,
    EnvironmentalAlert,
    SecurityAlert,
    QualityAlert,
    DeliveryConfirmation,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingQuery {
    pub product_id: Option<String>,
    pub company_id: Option<String>,
    pub status: Option<TrackingStatus>,
    pub location_type: Option<LocationType>,
    pub date_from: Option<u64>,
    pub date_to: Option<u64>,
    pub operator_id: Option<String>,
    pub shipment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupplyChainTrackingError {
    ProductNotFound,
    LocationNotFound,
    TrackingEntryNotFound,
    ShipmentNotFound,
    InvalidStatusTransition,
    InsufficientPermissions,
    InvalidLocation,
    InvalidStatus,
    DocumentNotFound,
    NotificationNotFound,
    EnvironmentalDataInvalid,
    JourneyNotFound,
}

impl SupplyChainTracking {
    pub fn new() -> Self {
        let mut system = Self {
            tracking_entries: HashMap::new(),
            product_journeys: HashMap::new(),
            location_registry: HashMap::new(),
            status_rules: HashMap::new(),
            notifications: HashMap::new(),
            shipments: HashMap::new(),
            role_system: RolePermissionSystem::new(),
        };

        system.initialize_default_status_rules();
        system
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

    // Initialize default status transition rules
    fn initialize_default_status_rules(&mut self) {
        let default_rules = vec![
            StatusTransitionRule {
                from_status: TrackingStatus::Created,
                to_status: TrackingStatus::InProduction,
                required_role: Some(UserRole::Manager),
                required_location_type: Some(LocationType::Factory),
                automatic: false,
                conditions: vec![TransitionCondition::UserConfirmation],
            },
            StatusTransitionRule {
                from_status: TrackingStatus::InProduction,
                to_status: TrackingStatus::QualityCheck,
                required_role: Some(UserRole::Warehouse),
                required_location_type: Some(LocationType::Factory),
                automatic: false,
                conditions: vec![TransitionCondition::DocumentUploaded("QualityReport".to_string())],
            },
            StatusTransitionRule {
                from_status: TrackingStatus::QualityCheck,
                to_status: TrackingStatus::ReadyToShip,
                required_role: Some(UserRole::Warehouse),
                required_location_type: Some(LocationType::Warehouse),
                automatic: false,
                conditions: vec![TransitionCondition::UserConfirmation],
            },
            StatusTransitionRule {
                from_status: TrackingStatus::ReadyToShip,
                to_status: TrackingStatus::InTransit,
                required_role: Some(UserRole::Transport),
                required_location_type: None,
                automatic: false,
                conditions: vec![TransitionCondition::UserConfirmation],
            },
            StatusTransitionRule {
                from_status: TrackingStatus::InTransit,
                to_status: TrackingStatus::Delivered,
                required_role: Some(UserRole::Transport),
                required_location_type: Some(LocationType::CustomerLocation),
                automatic: false,
                conditions: vec![TransitionCondition::LocationReached("destination".to_string())],
            },
        ];

        self.status_rules.insert("default".to_string(), default_rules);
    }

    // Register a location
    pub fn register_location(
        &mut self,
        user_id: String,
        company_id: String,
        name: String,
        address: String,
        coordinates: Option<Coordinates>,
        location_type: LocationType,
        timezone: String,
        contact_info: Option<ContactInfo>,
    ) -> Result<String, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::Settings, &ActionType::Create) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let location_id = format!("loc_{}", self.location_registry.len() + 1);

        let location = LocationInfo {
            id: location_id.clone(),
            name,
            address,
            coordinates,
            location_type,
            company_id: Some(company_id),
            timezone,
            contact_info,
        };

        self.location_registry.insert(location_id.clone(), location);
        Ok(location_id)
    }

    // Create a tracking entry
    pub fn create_tracking_entry(
        &mut self,
        user_id: String,
        company_id: String,
        product_id: String,
        status: TrackingStatus,
        location_id: String,
        notes: Option<String>,
        metadata: HashMap<String, String>,
        environmental_data: Option<EnvironmentalData>,
    ) -> Result<String, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Create) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        // Get location info
        let location = self.location_registry.get(&location_id)
            .ok_or(SupplyChainTrackingError::LocationNotFound)?
            .clone();

        // Validate status transition if product journey exists
        if let Some(journey) = self.product_journeys.get(&product_id) {
            let current_status = journey.current_status.clone();
            self.validate_status_transition(&current_status, &status, &user_id, &company_id, &location)?;
        }

        let entry_id = format!("track_{}", self.tracking_entries.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = TrackingEntry {
            id: entry_id.clone(),
            product_id: product_id.clone(),
            shipment_id: None,
            status: status.clone(),
            location: location.clone(),
            operator_id: user_id.clone(),
            company_id: company_id.clone(),
            timestamp: current_time,
            notes,
            metadata,
            documents: Vec::new(),
            environmental_data,
        };

        self.tracking_entries.insert(entry_id.clone(), entry);

        // Update or create product journey
        self.update_product_journey(product_id.clone(), entry_id.clone(), status.clone(), location, current_time)?;

        // Create notification for status change
        self.create_notification(
            product_id.clone(),
            NotificationType::StatusChange,
            format!("Product status updated to {:?}", status),
            NotificationSeverity::Info,
            vec![user_id],
        )?;

        Ok(entry_id)
    }

    // Update product journey
    fn update_product_journey(
        &mut self,
        product_id: String,
        entry_id: String,
        status: TrackingStatus,
        location: LocationInfo,
        timestamp: u64,
    ) -> Result<(), SupplyChainTrackingError> {
        if let Some(journey) = self.product_journeys.get_mut(&product_id) {
            journey.tracking_entries.push(entry_id);
            journey.current_status = status.clone();
            journey.current_location = location.clone();
            
            // Update delivery time if delivered
            if matches!(status, TrackingStatus::Delivered) {
                journey.actual_delivery = Some(timestamp);
            }

            // Add company to involved companies if not already present
            if let Some(company_id) = &location.company_id {
                if !journey.companies_involved.contains(company_id) {
                    journey.companies_involved.push(company_id.clone());
                }
            }
        } else {
            // Create new journey
            let journey = ProductJourney {
                product_id: product_id.clone(),
                tracking_entries: vec![entry_id],
                current_status: status,
                current_location: location.clone(),
                started_at: timestamp,
                estimated_delivery: None,
                actual_delivery: None,
                total_distance: None,
                companies_involved: if let Some(company_id) = location.company_id {
                    vec![company_id]
                } else {
                    Vec::new()
                },
            };

            self.product_journeys.insert(product_id, journey);
        }

        Ok(())
    }

    // Validate status transition
    fn validate_status_transition(
        &mut self,
        from_status: &TrackingStatus,
        to_status: &TrackingStatus,
        user_id: &str,
        company_id: &str,
        location: &LocationInfo,
    ) -> Result<(), SupplyChainTrackingError> {
        // Get status rules (using default for now)
        if let Some(rules) = self.status_rules.get("default") {
            for rule in rules {
                if rule.from_status == *from_status && rule.to_status == *to_status {
                    // Check role requirement
                    if let Some(required_role) = &rule.required_role {
                        if let Some(user_role) = self.role_system.get_user_role(company_id, user_id) {
                            if user_role != required_role && *user_role != UserRole::Owner {
                                return Err(SupplyChainTrackingError::InvalidStatusTransition);
                            }
                        } else {
                            return Err(SupplyChainTrackingError::InsufficientPermissions);
                        }
                    }

                    // Check location type requirement
                    if let Some(required_location_type) = &rule.required_location_type {
                        if location.location_type != *required_location_type {
                            return Err(SupplyChainTrackingError::InvalidLocation);
                        }
                    }

                    return Ok(());
                }
            }
        }

        // If no rule found, allow transition for owners and managers
        if let Some(user_role) = self.role_system.get_user_role(company_id, user_id) {
            if matches!(user_role, UserRole::Owner | UserRole::Manager) {
                return Ok(());
            }
        }

        Err(SupplyChainTrackingError::InvalidStatusTransition)
    }

    // Create shipment
    pub fn create_shipment(
        &mut self,
        user_id: String,
        company_id: String,
        product_ids: Vec<String>,
        origin_location_id: String,
        destination_location_id: String,
        carrier_company_id: String,
        estimated_delivery: Option<u64>,
        tracking_number: Option<String>,
        special_instructions: Option<String>,
    ) -> Result<String, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Create) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        // Get locations
        let origin = self.location_registry.get(&origin_location_id)
            .ok_or(SupplyChainTrackingError::LocationNotFound)?
            .clone();
        let destination = self.location_registry.get(&destination_location_id)
            .ok_or(SupplyChainTrackingError::LocationNotFound)?
            .clone();

        let shipment_id = format!("ship_{}", self.shipments.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let shipment = Shipment {
            id: shipment_id.clone(),
            product_ids: product_ids.clone(),
            origin,
            destination,
            carrier_company_id,
            driver_id: None,
            vehicle_id: None,
            status: ShipmentStatus::Pending,
            created_at: current_time,
            estimated_delivery,
            actual_delivery: None,
            tracking_number,
            route: vec![origin_location_id.clone(), destination_location_id],
            special_instructions,
        };

        self.shipments.insert(shipment_id.clone(), shipment);

        // Update tracking entries for products in shipment
        for product_id in product_ids {
            // Create tracking entry for shipment start
            self.create_tracking_entry(
                user_id.clone(),
                company_id.clone(),
                product_id.clone(),
                TrackingStatus::InTransit,
                origin_location_id.clone(),
                Some(format!("Added to shipment {}", shipment_id)),
                {
                    let mut metadata = HashMap::new();
                    metadata.insert("shipment_id".to_string(), shipment_id.clone());
                    metadata
                },
                None,
            )?;
        }

        Ok(shipment_id)
    }

    // Add document to tracking entry
    pub fn add_document_to_entry(
        &mut self,
        user_id: String,
        company_id: String,
        entry_id: String,
        document_type: DocumentType,
        name: String,
        hash: String,
    ) -> Result<String, SupplyChainTrackingError> {
        // Get entry to check permissions
        let entry = self.tracking_entries.get(&entry_id)
            .ok_or(SupplyChainTrackingError::TrackingEntryNotFound)?;

        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Update) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let doc_id = format!("doc_{}", entry.documents.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let document = TrackingDocument {
            id: doc_id.clone(),
            document_type,
            name,
            hash,
            uploaded_by: user_id,
            uploaded_at: current_time,
            verified: false,
        };

        // Add document to entry
        let entry = self.tracking_entries.get_mut(&entry_id).unwrap();
        entry.documents.push(document);

        Ok(doc_id)
    }

    // Create notification
    fn create_notification(
        &mut self,
        product_id: String,
        notification_type: NotificationType,
        message: String,
        severity: NotificationSeverity,
        recipients: Vec<String>,
    ) -> Result<String, SupplyChainTrackingError> {
        let notification_id = format!("notif_{}", 
            self.notifications.values().map(|v| v.len()).sum::<usize>() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let notification = TrackingNotification {
            id: notification_id.clone(),
            notification_type,
            product_id: product_id.clone(),
            message,
            severity,
            created_at: current_time,
            recipients,
            acknowledged_by: Vec::new(),
            resolved: false,
        };

        self.notifications
            .entry(product_id)
            .or_insert_with(Vec::new)
            .push(notification);

        Ok(notification_id)
    }

    // Query tracking entries
    pub fn query_tracking_entries(
        &mut self,
        user_id: String,
        company_id: String,
        query: TrackingQuery,
    ) -> Result<Vec<TrackingEntry>, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Read) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let mut results = Vec::new();

        for entry in self.tracking_entries.values() {
            // Apply filters
            if let Some(ref product_id) = query.product_id {
                if entry.product_id != *product_id {
                    continue;
                }
            }

            if let Some(ref filter_company_id) = query.company_id {
                if entry.company_id != *filter_company_id {
                    continue;
                }
            }

            if let Some(ref status) = query.status {
                if entry.status != *status {
                    continue;
                }
            }

            if let Some(ref location_type) = query.location_type {
                if entry.location.location_type != *location_type {
                    continue;
                }
            }

            if let Some(date_from) = query.date_from {
                if entry.timestamp < date_from {
                    continue;
                }
            }

            if let Some(date_to) = query.date_to {
                if entry.timestamp > date_to {
                    continue;
                }
            }

            results.push(entry.clone());
        }

        Ok(results)
    }

    // Get product journey
    pub fn get_product_journey(&self, product_id: &str) -> Option<&ProductJourney> {
        self.product_journeys.get(product_id)
    }

    // Get tracking statistics
    pub fn get_tracking_stats(&self, company_id: &str) -> TrackingStats {
        let company_entries: Vec<_> = self.tracking_entries.values()
            .filter(|entry| entry.company_id == company_id)
            .collect();

        let total_entries = company_entries.len();
        let active_shipments = self.shipments.values()
            .filter(|shipment| 
                shipment.carrier_company_id == company_id && 
                !matches!(shipment.status, ShipmentStatus::Delivered | ShipmentStatus::Failed | ShipmentStatus::Cancelled)
            )
            .count();

        let delivered_products = company_entries.iter()
            .filter(|entry| matches!(entry.status, TrackingStatus::Delivered))
            .count();

        let in_transit_products = company_entries.iter()
            .filter(|entry| matches!(entry.status, TrackingStatus::InTransit))
            .count();

        let total_notifications = self.notifications.values()
            .map(|notifs| notifs.len())
            .sum();

        TrackingStats {
            total_entries,
            active_shipments,
            delivered_products,
            in_transit_products,
            total_locations: self.location_registry.values()
                .filter(|loc| loc.company_id.as_deref() == Some(company_id))
                .count(),
            total_notifications,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingStats {
    pub total_entries: usize,
    pub active_shipments: usize,
    pub delivered_products: usize,
    pub in_transit_products: usize,
    pub total_locations: usize,
    pub total_notifications: usize,
}

impl Default for SupplyChainTracking {
    fn default() -> Self {
        Self::new()
    }
}