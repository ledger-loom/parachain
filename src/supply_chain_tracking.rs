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
pub struct AdvancedSearchFilter {
    // Basic filters
    pub product_ids: Option<Vec<String>>,
    pub company_ids: Option<Vec<String>>,
    pub statuses: Option<Vec<TrackingStatus>>,
    pub location_types: Option<Vec<LocationType>>,
    pub operator_ids: Option<Vec<String>>,
    pub shipment_ids: Option<Vec<String>>,
    
    // Text search
    pub search_text: Option<String>,
    pub search_in_notes: bool,
    pub search_in_metadata: bool,
    
    // Date filters
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub delivered_after: Option<u64>,
    pub delivered_before: Option<u64>,
    
    // Location filters
    pub origin_location: Option<String>,
    pub destination_location: Option<String>,
    pub current_location: Option<String>,
    pub location_name_contains: Option<String>,
    
    // Environmental data filters
    pub min_temperature: Option<f64>,
    pub max_temperature: Option<f64>,
    pub min_humidity: Option<f64>,
    pub max_humidity: Option<f64>,
    
    // Journey filters
    pub min_journey_duration: Option<u64>,
    pub max_journey_duration: Option<u64>,
    pub has_delays: Option<bool>,
    pub companies_involved: Option<Vec<String>>,
    
    // Metadata filters
    pub metadata_filters: HashMap<String, String>,
    pub has_documents: Option<bool>,
    pub document_types: Option<Vec<String>>,
    
    // Pagination and sorting
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    Timestamp,
    ProductId,
    Status,
    Location,
    Operator,
    Company,
    Temperature,
    Humidity,
    JourneyDuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub total_count: usize,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSearchFilter {
    pub name_contains: Option<String>,
    pub location_types: Option<Vec<LocationType>>,
    pub company_id: Option<String>,
    pub has_contact_info: Option<bool>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipmentSearchFilter {
    pub carrier_company_ids: Option<Vec<String>>,
    pub destination_company_ids: Option<Vec<String>>,
    pub statuses: Option<Vec<ShipmentStatus>>,
    pub priority_levels: Option<Vec<String>>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub estimated_delivery_after: Option<u64>,
    pub estimated_delivery_before: Option<u64>,
    pub route_contains_location: Option<String>,
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

    // Advanced search for tracking entries with full filtering, sorting, and pagination
    pub fn advanced_search_tracking_entries(
        &mut self,
        user_id: String,
        company_id: String,
        filter: AdvancedSearchFilter,
    ) -> Result<SearchResult<TrackingEntry>, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Read) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let mut results: Vec<TrackingEntry> = self.tracking_entries.values()
            .filter(|entry| self.apply_advanced_filters(entry, &filter))
            .cloned()
            .collect();

        // Apply sorting
        self.sort_tracking_entries(&mut results, &filter);

        let total_count = results.len();
        let page = filter.page.unwrap_or(1);
        let page_size = filter.page_size.unwrap_or(50);
        let total_pages = (total_count as f64 / page_size as f64).ceil() as u32;

        // Apply pagination
        let start_index = ((page - 1) * page_size) as usize;
        let end_index = (start_index + page_size as usize).min(total_count);
        let paginated_results = results[start_index..end_index].to_vec();

        Ok(SearchResult {
            items: paginated_results,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    // Apply advanced filters to a tracking entry
    fn apply_advanced_filters(&self, entry: &TrackingEntry, filter: &AdvancedSearchFilter) -> bool {
        // Basic ID filters
        if let Some(ref product_ids) = filter.product_ids {
            if !product_ids.contains(&entry.product_id) {
                return false;
            }
        }

        if let Some(ref company_ids) = filter.company_ids {
            if !company_ids.contains(&entry.company_id) {
                return false;
            }
        }

        if let Some(ref statuses) = filter.statuses {
            if !statuses.contains(&entry.status) {
                return false;
            }
        }

        if let Some(ref location_types) = filter.location_types {
            if !location_types.contains(&entry.location.location_type) {
                return false;
            }
        }

        if let Some(ref operator_ids) = filter.operator_ids {
            if !operator_ids.contains(&entry.operator_id) {
                return false;
            }
        }

        if let Some(ref shipment_ids) = filter.shipment_ids {
            if let Some(ref entry_shipment_id) = entry.shipment_id {
                if !shipment_ids.contains(entry_shipment_id) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Text search
        if let Some(ref search_text) = filter.search_text {
            let search_text_lower = search_text.to_lowercase();
            let mut found = false;

            // Search in product ID
            if entry.product_id.to_lowercase().contains(&search_text_lower) {
                found = true;
            }

            // Search in location name and address
            if entry.location.name.to_lowercase().contains(&search_text_lower) ||
               entry.location.address.to_lowercase().contains(&search_text_lower) {
                found = true;
            }

            // Search in notes if enabled
            if filter.search_in_notes {
                if let Some(ref notes) = entry.notes {
                    if notes.to_lowercase().contains(&search_text_lower) {
                        found = true;
                    }
                }
            }

            // Search in metadata if enabled
            if filter.search_in_metadata {
                for (key, value) in &entry.metadata {
                    if key.to_lowercase().contains(&search_text_lower) ||
                       value.to_lowercase().contains(&search_text_lower) {
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                return false;
            }
        }

        // Date filters
        if let Some(created_after) = filter.created_after {
            if entry.timestamp < created_after {
                return false;
            }
        }

        if let Some(created_before) = filter.created_before {
            if entry.timestamp > created_before {
                return false;
            }
        }

        // Location filters
        if let Some(ref location_name) = filter.location_name_contains {
            if !entry.location.name.to_lowercase().contains(&location_name.to_lowercase()) {
                return false;
            }
        }

        // Environmental data filters
        if let Some(ref env_data) = entry.environmental_data {
            if let Some(min_temp) = filter.min_temperature {
                if let Some(temp) = env_data.temperature {
                    if temp < min_temp {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            if let Some(max_temp) = filter.max_temperature {
                if let Some(temp) = env_data.temperature {
                    if temp > max_temp {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            if let Some(min_humidity) = filter.min_humidity {
                if let Some(humidity) = env_data.humidity {
                    if humidity < min_humidity {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            if let Some(max_humidity) = filter.max_humidity {
                if let Some(humidity) = env_data.humidity {
                    if humidity > max_humidity {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        } else {
            // If no environmental data but filters are set, exclude entry
            if filter.min_temperature.is_some() || filter.max_temperature.is_some() ||
               filter.min_humidity.is_some() || filter.max_humidity.is_some() {
                return false;
            }
        }

        // Metadata filters
        for (key, expected_value) in &filter.metadata_filters {
            if let Some(actual_value) = entry.metadata.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Document filters
        if let Some(has_documents) = filter.has_documents {
            let entry_has_documents = !entry.documents.is_empty();
            if has_documents != entry_has_documents {
                return false;
            }
        }

        if let Some(ref document_types) = filter.document_types {
            let entry_document_types: Vec<String> = entry.documents.iter()
                .map(|doc| format!("{:?}", doc.document_type))
                .collect();
            
            let has_required_type = document_types.iter()
                .any(|doc_type| entry_document_types.contains(doc_type));
            
            if !has_required_type {
                return false;
            }
        }

        true
    }

    // Sort tracking entries based on the specified criteria
    fn sort_tracking_entries(&self, entries: &mut Vec<TrackingEntry>, filter: &AdvancedSearchFilter) {
        if let Some(ref sort_field) = filter.sort_by {
            let ascending = matches!(filter.sort_order, Some(SortOrder::Ascending));

            entries.sort_by(|a, b| {
                let comparison = match sort_field {
                    SortField::Timestamp => a.timestamp.cmp(&b.timestamp),
                    SortField::ProductId => a.product_id.cmp(&b.product_id),
                    SortField::Status => format!("{:?}", a.status).cmp(&format!("{:?}", b.status)),
                    SortField::Location => a.location.name.cmp(&b.location.name),
                    SortField::Operator => a.operator_id.cmp(&b.operator_id),
                    SortField::Company => a.company_id.cmp(&b.company_id),
                    SortField::Temperature => {
                        let temp_a = a.environmental_data.as_ref()
                            .and_then(|e| e.temperature)
                            .unwrap_or(0.0);
                        let temp_b = b.environmental_data.as_ref()
                            .and_then(|e| e.temperature)
                            .unwrap_or(0.0);
                        temp_a.partial_cmp(&temp_b).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    SortField::Humidity => {
                        let humidity_a = a.environmental_data.as_ref()
                            .and_then(|e| e.humidity)
                            .unwrap_or(0.0);
                        let humidity_b = b.environmental_data.as_ref()
                            .and_then(|e| e.humidity)
                            .unwrap_or(0.0);
                        humidity_a.partial_cmp(&humidity_b).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    SortField::JourneyDuration => {
                        // Calculate journey duration by looking at product journey
                        let duration_a = self.calculate_journey_duration(&a.product_id);
                        let duration_b = self.calculate_journey_duration(&b.product_id);
                        duration_a.cmp(&duration_b)
                    },
                };

                if ascending { comparison } else { comparison.reverse() }
            });
        }
    }

    // Helper method to calculate journey duration
    fn calculate_journey_duration(&self, product_id: &str) -> u64 {
        if let Some(journey) = self.product_journeys.get(product_id) {
            if let Some(delivered_at) = journey.actual_delivery {
                delivered_at.saturating_sub(journey.started_at)
            } else {
                // Use current time if not delivered yet
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                current_time.saturating_sub(journey.started_at)
            }
        } else {
            0
        }
    }

    // Search locations with advanced filtering
    pub fn search_locations(
        &mut self,
        user_id: String,
        company_id: String,
        filter: LocationSearchFilter,
    ) -> Result<Vec<LocationInfo>, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Read) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let results: Vec<LocationInfo> = self.location_registry.values()
            .filter(|location| {
                // Company filter
                if let Some(ref filter_company_id) = filter.company_id {
                    if location.company_id.as_ref() != Some(filter_company_id) {
                        return false;
                    }
                }

                // Name contains filter
                if let Some(ref name_filter) = filter.name_contains {
                    if !location.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                        return false;
                    }
                }

                // Location types filter
                if let Some(ref types) = filter.location_types {
                    if !types.contains(&location.location_type) {
                        return false;
                    }
                }

                // Contact info filter
                if let Some(has_contact) = filter.has_contact_info {
                    let location_has_contact = location.contact_info.is_some();
                    if has_contact != location_has_contact {
                        return false;
                    }
                }

                // City filter (extract from address)
                if let Some(ref city) = filter.city {
                    if !location.address.to_lowercase().contains(&city.to_lowercase()) {
                        return false;
                    }
                }

                // Timezone filter
                if let Some(ref timezone) = filter.timezone {
                    if location.timezone != *timezone {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        Ok(results)
    }

    // Search shipments with advanced filtering
    pub fn search_shipments(
        &mut self,
        user_id: String,
        company_id: String,
        filter: ShipmentSearchFilter,
    ) -> Result<Vec<Shipment>, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Read) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let results: Vec<Shipment> = self.shipments.values()
            .filter(|shipment| {
                // Carrier company filter
                if let Some(ref carrier_companies) = filter.carrier_company_ids {
                    if !carrier_companies.contains(&shipment.carrier_company_id) {
                        return false;
                    }
                }

                // Destination company filter
                if let Some(ref dest_companies) = filter.destination_company_ids {
                    if let Some(ref dest_company_id) = shipment.destination.company_id {
                        if !dest_companies.contains(dest_company_id) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Status filter
                if let Some(ref statuses) = filter.statuses {
                    if !statuses.contains(&shipment.status) {
                        return false;
                    }
                }

                // Created date filters
                if let Some(created_after) = filter.created_after {
                    if shipment.created_at < created_after {
                        return false;
                    }
                }

                if let Some(created_before) = filter.created_before {
                    if shipment.created_at > created_before {
                        return false;
                    }
                }

                // Estimated delivery filters
                if let Some(delivery_after) = filter.estimated_delivery_after {
                    if let Some(estimated) = shipment.estimated_delivery {
                        if estimated < delivery_after {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                if let Some(delivery_before) = filter.estimated_delivery_before {
                    if let Some(estimated) = shipment.estimated_delivery {
                        if estimated > delivery_before {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Route contains location filter
                if let Some(ref location_name) = filter.route_contains_location {
                    let route_contains = shipment.route.iter()
                        .any(|location_id| {
                            if let Some(location) = self.location_registry.get(location_id) {
                                location.name.to_lowercase().contains(&location_name.to_lowercase())
                            } else {
                                false
                            }
                        });
                    
                    if !route_contains {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        Ok(results)
    }

    // Get aggregated search statistics
    pub fn get_search_statistics(
        &mut self,
        user_id: String,
        company_id: String,
        filter: AdvancedSearchFilter,
    ) -> Result<SearchStatistics, SupplyChainTrackingError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::SupplyChain, &ActionType::Read) {
            return Err(SupplyChainTrackingError::InsufficientPermissions);
        }

        let filtered_entries: Vec<&TrackingEntry> = self.tracking_entries.values()
            .filter(|entry| self.apply_advanced_filters(entry, &filter))
            .collect();

        let total_entries = filtered_entries.len();
        let unique_products = filtered_entries.iter()
            .map(|entry| entry.product_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let unique_companies = filtered_entries.iter()
            .map(|entry| entry.company_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let unique_locations = filtered_entries.iter()
            .map(|entry| entry.location.id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let status_breakdown = self.calculate_status_breakdown(&filtered_entries);
        let location_breakdown = self.calculate_location_breakdown(&filtered_entries);

        Ok(SearchStatistics {
            total_entries,
            unique_products,
            unique_companies,
            unique_locations,
            status_breakdown,
            location_breakdown,
        })
    }

    // Helper method to calculate status breakdown
    fn calculate_status_breakdown(&self, entries: &[&TrackingEntry]) -> HashMap<TrackingStatus, usize> {
        let mut breakdown = HashMap::new();
        for entry in entries {
            *breakdown.entry(entry.status.clone()).or_insert(0) += 1;
        }
        breakdown
    }

    // Helper method to calculate location breakdown
    fn calculate_location_breakdown(&self, entries: &[&TrackingEntry]) -> HashMap<LocationType, usize> {
        let mut breakdown = HashMap::new();
        for entry in entries {
            *breakdown.entry(entry.location.location_type.clone()).or_insert(0) += 1;
        }
        breakdown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatistics {
    pub total_entries: usize,
    pub unique_products: usize,
    pub unique_companies: usize,
    pub unique_locations: usize,
    pub status_breakdown: HashMap<TrackingStatus, usize>,
    pub location_breakdown: HashMap<LocationType, usize>,
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