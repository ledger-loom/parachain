use crate::types::*;
use crate::role_permissions::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductManagement {
    products: HashMap<String, Product>,
    categories: HashMap<String, ProductCategory>,
    company_products: HashMap<String, Vec<String>>, // company_id -> product_ids
    category_products: HashMap<String, Vec<String>>, // category_id -> product_ids
    product_templates: HashMap<String, ProductTemplate>,
    product_batches: HashMap<String, ProductBatch>,
    role_system: RolePermissionSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_category_id: Option<String>,
    pub subcategories: Vec<String>,
    pub company_id: String,
    pub attributes_schema: Vec<AttributeSchema>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSchema {
    pub name: String,
    pub attribute_type: AttributeType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeType {
    Text,
    Number,
    Boolean,
    Date,
    Choice(Vec<String>), // Predefined choices
    Measurement { unit: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String), // Regex pattern
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category_id: String,
    pub company_id: String,
    pub default_attributes: HashMap<String, String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductBatch {
    pub id: String,
    pub product_id: String,
    pub batch_number: String,
    pub quantity: u32,
    pub manufacturing_date: u64,
    pub expiry_date: Option<u64>,
    pub quality_metrics: HashMap<String, String>,
    pub status: BatchStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchStatus {
    InProduction,
    QualityCheck,
    Approved,
    Rejected,
    Shipped,
    Delivered,
    Recalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSearchFilter {
    pub company_id: Option<String>,
    pub category_id: Option<String>,
    pub name_contains: Option<String>,
    pub attributes: HashMap<String, String>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductManagementError {
    ProductNotFound,
    CategoryNotFound,
    CompanyNotFound,
    UserNotFound,
    InsufficientPermissions,
    InvalidProductData,
    InvalidCategoryData,
    AttributeValidationError(String),
    DuplicateProduct,
    DuplicateCategory,
    CategoryHasProducts,
    TemplateNotFound,
    BatchNotFound,
}

impl ProductManagement {
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
            categories: HashMap::new(),
            company_products: HashMap::new(),
            category_products: HashMap::new(),
            product_templates: HashMap::new(),
            product_batches: HashMap::new(),
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

    // Create a new product category
    pub fn create_category(
        &mut self,
        user_id: String,
        company_id: String,
        name: String,
        description: String,
        parent_category_id: Option<String>,
        attributes_schema: Vec<AttributeSchema>,
    ) -> Result<String, ProductManagementError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::Product, &ActionType::Create) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        // Validate category name
        if name.trim().is_empty() {
            return Err(ProductManagementError::InvalidCategoryData);
        }

        // Check if parent category exists (if specified)
        if let Some(ref parent_id) = parent_category_id {
            if !self.categories.contains_key(parent_id) {
                return Err(ProductManagementError::CategoryNotFound);
            }
        }

        // Generate category ID
        let category_id = format!("cat_{}", self.categories.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create category
        let category = ProductCategory {
            id: category_id.clone(),
            name,
            description,
            parent_category_id: parent_category_id.clone(),
            subcategories: Vec::new(),
            company_id: company_id.clone(),
            attributes_schema,
            created_at: current_time,
        };

        // Update parent category's subcategories
        if let Some(parent_id) = parent_category_id {
            if let Some(parent_category) = self.categories.get_mut(&parent_id) {
                parent_category.subcategories.push(category_id.clone());
            }
        }

        self.categories.insert(category_id.clone(), category);
        Ok(category_id)
    }

    // Create a new product
    pub fn create_product(
        &mut self,
        user_id: String,
        company_id: String,
        name: String,
        description: String,
        category_id: String,
        attributes: HashMap<String, String>,
    ) -> Result<String, ProductManagementError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::Product, &ActionType::Create) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        // Validate product data
        if name.trim().is_empty() {
            return Err(ProductManagementError::InvalidProductData);
        }

        // Check if category exists
        let category = self.categories.get(&category_id)
            .ok_or(ProductManagementError::CategoryNotFound)?;

        // Validate attributes against category schema
        self.validate_attributes(&attributes, &category.attributes_schema)?;

        // Generate product ID
        let product_id = format!("prod_{}", self.products.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create product
        let product = Product {
            id: product_id.clone(),
            name,
            description,
            category: category_id.clone(),
            company_id: company_id.clone(),
            attributes,
            created_at: current_time,
            updated_at: current_time,
        };

        // Store product
        self.products.insert(product_id.clone(), product);

        // Update company products
        self.company_products
            .entry(company_id)
            .or_insert_with(Vec::new)
            .push(product_id.clone());

        // Update category products
        self.category_products
            .entry(category_id)
            .or_insert_with(Vec::new)
            .push(product_id.clone());

        Ok(product_id)
    }

    // Update product information
    pub fn update_product(
        &mut self,
        user_id: String,
        product_id: String,
        name: Option<String>,
        description: Option<String>,
        attributes: Option<HashMap<String, String>>,
    ) -> Result<(), ProductManagementError> {
        // Get product
        let product = self.products.get(&product_id)
            .ok_or(ProductManagementError::ProductNotFound)?;

        // Check permissions
        if !self.role_system.has_permission(&user_id, &product.company_id, &ResourceType::Product, &ActionType::Update) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        // Get category for attribute validation
        let category = self.categories.get(&product.category)
            .ok_or(ProductManagementError::CategoryNotFound)?;

        // Validate new attributes if provided
        if let Some(ref new_attributes) = attributes {
            self.validate_attributes(new_attributes, &category.attributes_schema)?;
        }

        // Update product
        let product = self.products.get_mut(&product_id).unwrap();
        
        if let Some(name) = name {
            if name.trim().is_empty() {
                return Err(ProductManagementError::InvalidProductData);
            }
            product.name = name;
        }

        if let Some(description) = description {
            product.description = description;
        }

        if let Some(attributes) = attributes {
            product.attributes = attributes;
        }

        product.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    // Create product template
    pub fn create_product_template(
        &mut self,
        user_id: String,
        company_id: String,
        name: String,
        description: String,
        category_id: String,
        default_attributes: HashMap<String, String>,
    ) -> Result<String, ProductManagementError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::Product, &ActionType::Create) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        // Validate template data
        if name.trim().is_empty() {
            return Err(ProductManagementError::InvalidProductData);
        }

        // Check if category exists
        let category = self.categories.get(&category_id)
            .ok_or(ProductManagementError::CategoryNotFound)?;

        // Validate default attributes
        self.validate_attributes(&default_attributes, &category.attributes_schema)?;

        // Generate template ID
        let template_id = format!("template_{}", self.product_templates.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create template
        let template = ProductTemplate {
            id: template_id.clone(),
            name,
            description,
            category_id,
            company_id,
            default_attributes,
            created_at: current_time,
        };

        self.product_templates.insert(template_id.clone(), template);
        Ok(template_id)
    }

    // Create product from template
    pub fn create_product_from_template(
        &mut self,
        user_id: String,
        template_id: String,
        name: String,
        description: Option<String>,
        attribute_overrides: HashMap<String, String>,
    ) -> Result<String, ProductManagementError> {
        // Get template
        let template = self.product_templates.get(&template_id)
            .ok_or(ProductManagementError::TemplateNotFound)?;

        // Merge default attributes with overrides
        let mut attributes = template.default_attributes.clone();
        attributes.extend(attribute_overrides);

        // Create product using template data
        self.create_product(
            user_id,
            template.company_id.clone(),
            name,
            description.unwrap_or_else(|| template.description.clone()),
            template.category_id.clone(),
            attributes,
        )
    }

    // Create product batch
    pub fn create_product_batch(
        &mut self,
        user_id: String,
        product_id: String,
        batch_number: String,
        quantity: u32,
        manufacturing_date: u64,
        expiry_date: Option<u64>,
        quality_metrics: HashMap<String, String>,
    ) -> Result<String, ProductManagementError> {
        // Get product
        let product = self.products.get(&product_id)
            .ok_or(ProductManagementError::ProductNotFound)?;

        // Check permissions
        if !self.role_system.has_permission(&user_id, &product.company_id, &ResourceType::Product, &ActionType::Create) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        // Generate batch ID
        let batch_id = format!("batch_{}", self.product_batches.len() + 1);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create batch
        let batch = ProductBatch {
            id: batch_id.clone(),
            product_id,
            batch_number,
            quantity,
            manufacturing_date,
            expiry_date,
            quality_metrics,
            status: BatchStatus::InProduction,
            created_at: current_time,
        };

        self.product_batches.insert(batch_id.clone(), batch);
        Ok(batch_id)
    }

    // Update batch status
    pub fn update_batch_status(
        &mut self,
        user_id: String,
        batch_id: String,
        status: BatchStatus,
    ) -> Result<(), ProductManagementError> {
        // Get batch
        let batch = self.product_batches.get(&batch_id)
            .ok_or(ProductManagementError::BatchNotFound)?;

        // Get product to check company
        let product = self.products.get(&batch.product_id)
            .ok_or(ProductManagementError::ProductNotFound)?;

        // Check permissions
        if !self.role_system.has_permission(&user_id, &product.company_id, &ResourceType::Product, &ActionType::Update) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        // Update batch status
        let batch = self.product_batches.get_mut(&batch_id).unwrap();
        batch.status = status;

        Ok(())
    }

    // Search products
    pub fn search_products(
        &mut self,
        user_id: String,
        company_id: String,
        filter: ProductSearchFilter,
    ) -> Result<Vec<Product>, ProductManagementError> {
        // Check permissions
        if !self.role_system.has_permission(&user_id, &company_id, &ResourceType::Product, &ActionType::Read) {
            return Err(ProductManagementError::InsufficientPermissions);
        }

        let mut results = Vec::new();

        for product in self.products.values() {
            // Check company filter
            if let Some(ref filter_company_id) = filter.company_id {
                if product.company_id != *filter_company_id {
                    continue;
                }
            }

            // Check category filter
            if let Some(ref filter_category_id) = filter.category_id {
                if product.category != *filter_category_id {
                    continue;
                }
            }

            // Check name filter
            if let Some(ref name_contains) = filter.name_contains {
                if !product.name.to_lowercase().contains(&name_contains.to_lowercase()) {
                    continue;
                }
            }

            // Check attribute filters
            let mut matches_attributes = true;
            for (key, value) in &filter.attributes {
                if let Some(product_value) = product.attributes.get(key) {
                    if product_value != value {
                        matches_attributes = false;
                        break;
                    }
                } else {
                    matches_attributes = false;
                    break;
                }
            }
            if !matches_attributes {
                continue;
            }

            // Check date filters
            if let Some(created_after) = filter.created_after {
                if product.created_at < created_after {
                    continue;
                }
            }

            if let Some(created_before) = filter.created_before {
                if product.created_at > created_before {
                    continue;
                }
            }

            results.push(product.clone());
        }

        Ok(results)
    }

    // Validate attributes against schema
    fn validate_attributes(
        &self,
        attributes: &HashMap<String, String>,
        schema: &[AttributeSchema],
    ) -> Result<(), ProductManagementError> {
        for attribute_schema in schema {
            if attribute_schema.required {
                if !attributes.contains_key(&attribute_schema.name) {
                    return Err(ProductManagementError::AttributeValidationError(
                        format!("Required attribute '{}' is missing", attribute_schema.name)
                    ));
                }
            }

            if let Some(value) = attributes.get(&attribute_schema.name) {
                // Validate based on attribute type
                match &attribute_schema.attribute_type {
                    AttributeType::Number => {
                        if value.parse::<f64>().is_err() {
                            return Err(ProductManagementError::AttributeValidationError(
                                format!("Attribute '{}' must be a number", attribute_schema.name)
                            ));
                        }
                    },
                    AttributeType::Boolean => {
                        if !matches!(value.as_str(), "true" | "false") {
                            return Err(ProductManagementError::AttributeValidationError(
                                format!("Attribute '{}' must be true or false", attribute_schema.name)
                            ));
                        }
                    },
                    AttributeType::Choice(choices) => {
                        if !choices.contains(value) {
                            return Err(ProductManagementError::AttributeValidationError(
                                format!("Attribute '{}' must be one of: {:?}", attribute_schema.name, choices)
                            ));
                        }
                    },
                    _ => {}, // Text, Date, Measurement - basic validation passed
                }

                // Apply validation rules
                for rule in &attribute_schema.validation_rules {
                    match rule {
                        ValidationRule::MinLength(min_len) => {
                            if value.len() < *min_len {
                                return Err(ProductManagementError::AttributeValidationError(
                                    format!("Attribute '{}' must be at least {} characters", attribute_schema.name, min_len)
                                ));
                            }
                        },
                        ValidationRule::MaxLength(max_len) => {
                            if value.len() > *max_len {
                                return Err(ProductManagementError::AttributeValidationError(
                                    format!("Attribute '{}' must be at most {} characters", attribute_schema.name, max_len)
                                ));
                            }
                        },
                        ValidationRule::MinValue(min_val) => {
                            if let Ok(num_value) = value.parse::<f64>() {
                                if num_value < *min_val {
                                    return Err(ProductManagementError::AttributeValidationError(
                                        format!("Attribute '{}' must be at least {}", attribute_schema.name, min_val)
                                    ));
                                }
                            }
                        },
                        ValidationRule::MaxValue(max_val) => {
                            if let Ok(num_value) = value.parse::<f64>() {
                                if num_value > *max_val {
                                    return Err(ProductManagementError::AttributeValidationError(
                                        format!("Attribute '{}' must be at most {}", attribute_schema.name, max_val)
                                    ));
                                }
                            }
                        },
                        ValidationRule::Required => {}, // Already checked above
                        ValidationRule::Pattern(_pattern) => {
                            // Regex validation would go here in a real implementation
                            // For demo purposes, we'll skip complex regex validation
                        },
                    }
                }
            }
        }

        Ok(())
    }

    // Get product by ID
    pub fn get_product(&self, product_id: &str) -> Option<&Product> {
        self.products.get(product_id)
    }

    // Get category by ID
    pub fn get_category(&self, category_id: &str) -> Option<&ProductCategory> {
        self.categories.get(category_id)
    }

    // Get products by company
    pub fn get_company_products(&self, company_id: &str) -> Vec<&Product> {
        if let Some(product_ids) = self.company_products.get(company_id) {
            product_ids.iter()
                .filter_map(|id| self.products.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    // Get products by category
    pub fn get_category_products(&self, category_id: &str) -> Vec<&Product> {
        if let Some(product_ids) = self.category_products.get(category_id) {
            product_ids.iter()
                .filter_map(|id| self.products.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    // Get product statistics
    pub fn get_product_stats(&self, company_id: &str) -> ProductStats {
        let company_products = self.get_company_products(company_id);
        let total_products = company_products.len();
        
        let categories_used = company_products.iter()
            .map(|product| &product.category)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let total_batches = self.product_batches.values()
            .filter(|batch| {
                if let Some(product) = self.products.get(&batch.product_id) {
                    product.company_id == company_id
                } else {
                    false
                }
            })
            .count();

        ProductStats {
            total_products,
            total_categories: self.categories.values()
                .filter(|cat| cat.company_id == company_id)
                .count(),
            categories_used,
            total_batches,
            total_templates: self.product_templates.values()
                .filter(|template| template.company_id == company_id)
                .count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductStats {
    pub total_products: usize,
    pub total_categories: usize,
    pub categories_used: usize,
    pub total_batches: usize,
    pub total_templates: usize,
}

impl Default for ProductManagement {
    fn default() -> Self {
        Self::new()
    }
}