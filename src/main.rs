mod types;
mod user_management;
mod company_management;
mod role_permissions;
mod product_management;
mod supply_chain_tracking;

use types::*;
use user_management::*;
use company_management::*;
use role_permissions::*;
use product_management::*;
use supply_chain_tracking::*;

fn main() {
    println!("🚀 Supply Chain Parachain Node Starting...");
    println!("📦 Environment: Development");
    println!("🔧 Status: Supply Chain Tracking pallet implemented");
    println!("📋 Next: Creating frontend web application");
    println!();
    
    // Demo the core data structures
    demo_core_structures();
    
    // Demo user management functionality
    demo_user_management();
    
    // Demo company management functionality
    demo_company_management();
    
    // Demo role & permissions system
    demo_role_permissions();
    
    // Demo product management system
    demo_product_management();
    
    // Demo supply chain tracking system
    demo_supply_chain_tracking();
    
    println!("Development environment setup completed successfully!");
}

fn demo_core_structures() {
    println!("🏗️  Demonstrating Core Data Structures:");
    
    // Create a sample user
    let user = User {
        id: "user_001".to_string(),
        email: "alice@example.com".to_string(),
        wallet_address: Some("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY".to_string()),
        profile: UserProfile {
            name: "Alice Smith".to_string(),
            company_id: Some("company_001".to_string()),
            role: Some(UserRole::Manager),
            settings: std::collections::HashMap::new(),
        },
        verified: true,
        created_at: 1640995200, // Unix timestamp
    };
    println!("   👤 User: {} ({})", user.profile.name, user.email);
    
    // Create a sample company
    let company = Company {
        id: "company_001".to_string(),
        name: "Supply Chain Solutions Inc".to_string(),
        description: "Leading supply chain management company".to_string(),
        owner_id: user.id.clone(),
        members: vec![CompanyMember {
            user_id: user.id.clone(),
            role: UserRole::Owner,
            added_at: 1640995200,
        }],
        settings: CompanySettings {
            industry: "Logistics".to_string(),
            location: "Global".to_string(),
            preferences: std::collections::HashMap::new(),
        },
        created_at: 1640995200,
    };
    println!("   🏢 Company: {}", company.name);
    
    // Create a sample product
    let product = Product {
        id: "prod_001".to_string(),
        name: "Organic Coffee Beans".to_string(),
        description: "Premium organic coffee beans from Ethiopia".to_string(),
        category: "Food & Beverage".to_string(),
        company_id: company.id.clone(),
        attributes: {
            let mut attrs = std::collections::HashMap::new();
            attrs.insert("origin".to_string(), "Ethiopia".to_string());
            attrs.insert("weight".to_string(), "1kg".to_string());
            attrs.insert("certification".to_string(), "Organic".to_string());
            attrs
        },
        created_at: 1640995200,
        updated_at: 1640995200,
    };
    println!("   📦 Product: {}", product.name);
    
    // Create a sample supply chain entry
    let supply_entry = SupplyChainEntry {
        id: "entry_001".to_string(),
        product_id: product.id.clone(),
        status: ProductStatus::Manufactured,
        location: Location {
            name: "Coffee Processing Plant".to_string(),
            address: "Addis Ababa, Ethiopia".to_string(),
            coordinates: Some(Coordinates {
                latitude: 9.0320,
                longitude: 38.7441,
            }),
            company_id: Some(company.id.clone()),
        },
        timestamp: 1640995200,
        operator_id: user.id.clone(),
        notes: Some("Quality check passed".to_string()),
        metadata: std::collections::HashMap::new(),
    };
    println!("   🚚 Supply Chain Entry: {} at {}", 
             format!("{:?}", supply_entry.status), 
             supply_entry.location.name);
    
    println!("   ✅ All core data structures working correctly!");
}

fn demo_user_management() {
    println!("\n👤 Demonstrating User Management Pallet:");
    
    let mut user_mgmt = UserManagement::new();
    
    // Register a new user
    let user_id = user_mgmt.register_user(
        "alice@example.com".to_string(),
        "hashed_password_123".to_string(),
        "Alice Smith".to_string(),
        Some("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY".to_string()),
    ).expect("Failed to register user");
    
    println!("   📝 User registered: {}", user_id);
    
    // Get user information
    let user = user_mgmt.get_user(&user_id).expect("User should exist");
    println!("   👤 User info: {} ({}) - Verified: {}", 
             user.profile.name, user.email, user.verified);
    
    // Submit verification request
    let documents = vec![
        Document {
            id: "doc_001".to_string(),
            name: "Driver's License".to_string(),
            document_type: user_management::DocumentType::GovernmentId,
            hash: "abc123hash".to_string(),
            uploaded_at: 1640995200,
        }
    ];
    
    let verification_id = user_mgmt.submit_verification(
        &user_id,
        VerificationType::Identity,
        documents,
    ).expect("Failed to submit verification");
    
    println!("   📋 Verification submitted: {}", verification_id);
    
    // Review and approve verification (admin action)
    user_mgmt.review_verification(
        &verification_id,
        VerificationStatus::Approved,
        Some("Identity documents verified successfully".to_string()),
    ).expect("Failed to review verification");
    
    println!("   ✅ Verification approved");
    
    // Check user verification status
    let is_verified = user_mgmt.get_verification_status(&user_id).unwrap_or(false);
    println!("   🔒 User verification status: {}", is_verified);
    
    // Register another user
    let user_id2 = user_mgmt.register_user(
        "bob@example.com".to_string(),
        "hashed_password_456".to_string(),
        "Bob Johnson".to_string(),
        None, // No wallet initially
    ).expect("Failed to register second user");
    
    // Link wallet to second user
    user_mgmt.link_wallet(&user_id2, "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".to_string())
        .expect("Failed to link wallet");
    
    println!("   🔗 Wallet linked for user: {}", user_id2);
    
    // Update user profile
    let mut new_settings = std::collections::HashMap::new();
    new_settings.insert("notification_email".to_string(), "true".to_string());
    new_settings.insert("preferred_language".to_string(), "en".to_string());
    
    user_mgmt.update_user_profile(&user_id, None, Some(new_settings))
        .expect("Failed to update profile");
    
    println!("   ⚙️  User profile updated");
    
    // Get user statistics
    let stats = user_mgmt.get_user_stats();
    println!("   📊 User Statistics:");
    println!("      - Total Users: {}", stats.total_users);
    println!("      - Verified Users: {}", stats.verified_users);
    println!("      - Unverified Users: {}", stats.unverified_users);
    println!("      - Pending Verifications: {}", stats.pending_verifications);
    
    // Test authentication
    let auth_user = user_mgmt.authenticate_user("alice@example.com", "hashed_password_123");
    match auth_user {
        Ok(user) => println!("   🔐 Authentication successful for: {}", user.profile.name),
        Err(_) => println!("   ❌ Authentication failed"),
    }
    
    // Test lookup by wallet
    let wallet_user = user_mgmt.get_user_by_wallet("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY");
    if let Some(user) = wallet_user {
        println!("   🪙 User found by wallet: {}", user.profile.name);
    }
    
    println!("   ✅ User Management pallet working correctly!");
}

fn demo_company_management() {
    println!("\n🏢 Demonstrating Company Management Pallet:");
    
    let mut company_mgmt = CompanyManagement::new();
    
    // Create a company
    let company_id = company_mgmt.create_company(
        "user_1".to_string(),
        "Supply Chain Solutions Inc".to_string(),
        "Leading supply chain management company".to_string(),
        "Logistics".to_string(),
        "Global".to_string(),
    ).expect("Failed to create company");
    
    println!("   🏭 Company created: {}", company_id);
    
    // Get company information
    let company = company_mgmt.get_company(&company_id).expect("Company should exist");
    println!("   🏢 Company: {} - Members: {}", company.name, company.members.len());
    
    // Invite a user to join the company
    let invitation_id = company_mgmt.invite_user(
        company_id.clone(),
        "user_1".to_string(),
        "bob@example.com".to_string(),
        UserRole::Manager,
        Some("Welcome to our supply chain team!".to_string()),
    ).expect("Failed to send invitation");
    
    println!("   📧 Invitation sent: {}", invitation_id);
    
    // Accept the invitation
    company_mgmt.accept_invitation(invitation_id, "user_2".to_string())
        .expect("Failed to accept invitation");
    
    println!("   ✅ Invitation accepted by user_2");
    
    // Check updated member count
    let updated_company = company_mgmt.get_company(&company_id).expect("Company should exist");
    println!("   👥 Updated member count: {}", updated_company.members.len());
    
    // Update company settings
    let mut preferences = std::collections::HashMap::new();
    preferences.insert("timezone".to_string(), "UTC".to_string());
    preferences.insert("currency".to_string(), "USD".to_string());
    
    company_mgmt.update_company_settings(
        company_id.clone(),
        "user_1".to_string(),
        Some("Supply Chain Solutions International".to_string()),
        None,
        None,
        Some("North America".to_string()),
        Some(preferences),
    ).expect("Failed to update company settings");
    
    println!("   ⚙️  Company settings updated");
    
    // Get user's companies
    let user_companies = company_mgmt.get_user_companies("user_1");
    println!("   👤 User's companies: {}", user_companies.len());
    
    // Create another user invitation
    let invitation_id2 = company_mgmt.invite_user(
        company_id.clone(),
        "user_1".to_string(),
        "charlie@example.com".to_string(),
        UserRole::Warehouse,
        None,
    ).expect("Failed to send second invitation");
    
    // Decline the invitation
    company_mgmt.decline_invitation(invitation_id2)
        .expect("Failed to decline invitation");
    
    println!("   ❌ Second invitation declined");
    
    // Get company invitations
    let pending_invites = company_mgmt.get_company_invitations(&company_id);
    println!("   📋 Pending invitations: {}", pending_invites.len());
    
    // Request ownership transfer
    let transfer_id = company_mgmt.request_ownership_transfer(
        company_id.clone(),
        "user_1".to_string(),
        "user_2".to_string(),
        Some("Retiring from active management".to_string()),
    ).expect("Failed to transfer ownership");
    
    println!("   👑 Ownership transferred: {}", transfer_id);
    
    // Verify new ownership
    let final_company = company_mgmt.get_company(&company_id).expect("Company should exist");
    println!("   👤 New owner: {}", final_company.owner_id);
    
    // Get company statistics
    let stats = company_mgmt.get_company_stats();
    println!("   📊 Company Statistics:");
    println!("      - Total Companies: {}", stats.total_companies);
    println!("      - Total Members: {}", stats.total_members);
    println!("      - Pending Invitations: {}", stats.pending_invitations);
    println!("      - Avg Members per Company: {:.1}", stats.average_members_per_company);
    
    println!("   ✅ Company Management pallet working correctly!");
}

fn demo_role_permissions() {
    println!("\n🔐 Demonstrating Role & Permissions System:");
    
    let mut role_system = RolePermissionSystem::new();
    
    // Assign roles to users in a company
    let company_id = "company_1".to_string();
    let owner_id = "user_1".to_string();
    let manager_id = "user_2".to_string();
    let warehouse_id = "user_3".to_string();
    
    // Set owner role (simulates company creation)
    role_system.set_owner_role(company_id.clone(), owner_id.clone());
    
    // Owner assigns manager role
    role_system.assign_role(company_id.clone(), manager_id.clone(), UserRole::Manager, owner_id.clone())
        .expect("Failed to assign manager role");
    
    println!("   👑 Owner role: {} assigned to company {}", owner_id, company_id);
    println!("   👔 Manager role assigned to user: {}", manager_id);
    
    // Owner assigns warehouse role (managers can manage their subordinates)
    role_system.assign_role(company_id.clone(), warehouse_id.clone(), UserRole::Warehouse, owner_id.clone())
        .expect("Failed to assign warehouse role");
    
    println!("   📦 Warehouse role assigned to user: {}", warehouse_id);
    
    // Test permission checks
    println!("\n   🔍 Testing Permission Checks:");
    
    // Owner should have all permissions
    let can_manage_company = role_system.has_permission(&owner_id, &company_id, &ResourceType::Company, &ActionType::Manage);
    println!("      Owner can manage company: {}", can_manage_company);
    
    let can_create_products = role_system.has_permission(&manager_id, &company_id, &ResourceType::Product, &ActionType::Create);
    println!("      Manager can create products: {}", can_create_products);
    
    let can_delete_company = role_system.has_permission(&warehouse_id, &company_id, &ResourceType::Company, &ActionType::Delete);
    println!("      Warehouse can delete company: {}", can_delete_company);
    
    // Grant custom permission
    role_system.grant_custom_permission(
        company_id.clone(),
        warehouse_id.clone(),
        ResourceType::Reports,
        ActionType::Export,
        owner_id.clone(),
    ).expect("Failed to grant custom permission");
    
    println!("   ➕ Custom permission granted: Warehouse can export reports");
    
    // Test custom permission
    let can_export_reports = role_system.has_permission(&warehouse_id, &company_id, &ResourceType::Reports, &ActionType::Export);
    println!("      Warehouse can export reports (custom): {}", can_export_reports);
    
    // Get all user permissions
    let manager_permissions = role_system.get_user_permissions(&company_id, &manager_id);
    println!("   📋 Manager has {} permissions", manager_permissions.len());
    
    // Get users with specific role
    let managers = role_system.get_users_with_role(&company_id, &UserRole::Manager);
    println!("   👥 Users with Manager role: {}", managers.len());
    
    // Test permission denial
    let can_transfer_ownership = role_system.has_permission(&manager_id, &company_id, &ResourceType::Company, &ActionType::Transfer);
    println!("      Manager can transfer ownership: {}", can_transfer_ownership);
    
    // Get audit logs
    let owner_logs = role_system.get_user_audit_logs(&owner_id);
    println!("   📊 Owner audit log entries: {}", owner_logs.len());
    
    let company_logs = role_system.get_company_audit_logs(&company_id);
    println!("   📊 Company audit log entries: {}", company_logs.len());
    
    // Revoke custom permission
    role_system.revoke_custom_permission(
        company_id.clone(),
        warehouse_id.clone(),
        ResourceType::Reports,
        ActionType::Export,
        owner_id.clone(),
    ).expect("Failed to revoke custom permission");
    
    println!("   ➖ Custom permission revoked");
    
    // Test revoked permission
    let can_still_export = role_system.has_permission(&warehouse_id, &company_id, &ResourceType::Reports, &ActionType::Export);
    println!("      Warehouse can still export reports: {}", can_still_export);
    
    // Get permission statistics
    let stats = role_system.get_permission_stats();
    println!("   📈 Permission Statistics:");
    println!("      - Role Assignments: {}", stats.total_role_assignments);
    println!("      - Custom Permissions: {}", stats.total_custom_permissions);
    println!("      - Audit Entries: {}", stats.total_audit_entries);
    println!("      - Granted Permissions: {}", stats.granted_permissions);
    println!("      - Denied Permissions: {}", stats.denied_permissions);
    
    // Test different role capabilities
    println!("\n   🎭 Role Capability Summary:");
    
    let roles = [
        (&owner_id, "Owner"),
        (&manager_id, "Manager"),
        (&warehouse_id, "Warehouse"),
    ];
    
    for (user_id, role_name) in roles.iter() {
        let permissions = role_system.get_user_permissions(&company_id, user_id);
        let can_create_products = role_system.has_permission(user_id, &company_id, &ResourceType::Product, &ActionType::Create);
        let can_manage_users = role_system.has_permission(user_id, &company_id, &ResourceType::User, &ActionType::Manage);
        let can_read_reports = role_system.has_permission(user_id, &company_id, &ResourceType::Reports, &ActionType::Read);
        
        println!("      {} ({}): {} perms, Products: {}, Users: {}, Reports: {}", 
                 role_name, user_id, permissions.len(), can_create_products, can_manage_users, can_read_reports);
    }
    
    println!("   ✅ Role & Permissions system working correctly!");
}

fn demo_product_management() {
    println!("\n📦 Demonstrating Product Management Pallet:");
    
    let mut product_mgmt = ProductManagement::new();
    let company_id = "company_1".to_string();
    let owner_id = "user_1".to_string();
    let manager_id = "user_2".to_string();
    
    // Set up roles first
    product_mgmt.set_owner_role(company_id.clone(), owner_id.clone());
    product_mgmt.assign_role(company_id.clone(), manager_id.clone(), UserRole::Manager, owner_id.clone())
        .expect("Failed to assign manager role");
    
    // Create product categories with attribute schemas
    let food_category_schema = vec![
        AttributeSchema {
            name: "origin".to_string(),
            attribute_type: AttributeType::Text,
            required: true,
            default_value: None,
            validation_rules: vec![ValidationRule::MinLength(2)],
        },
        AttributeSchema {
            name: "weight".to_string(),
            attribute_type: AttributeType::Measurement { unit: "kg".to_string() },
            required: true,
            default_value: Some("1.0".to_string()),
            validation_rules: vec![ValidationRule::MinValue(0.1), ValidationRule::MaxValue(100.0)],
        },
        AttributeSchema {
            name: "organic".to_string(),
            attribute_type: AttributeType::Boolean,
            required: false,
            default_value: Some("false".to_string()),
            validation_rules: vec![],
        },
        AttributeSchema {
            name: "grade".to_string(),
            attribute_type: AttributeType::Choice(vec!["A".to_string(), "B".to_string(), "C".to_string()]),
            required: false,
            default_value: Some("B".to_string()),
            validation_rules: vec![],
        },
    ];
    
    let category_id = product_mgmt.create_category(
        owner_id.clone(),
        company_id.clone(),
        "Food & Beverages".to_string(),
        "Food and beverage products".to_string(),
        None,
        food_category_schema.clone(),
    ).expect("Failed to create category");
    
    println!("   🏷️  Category created: {}", category_id);
    
    // Create subcategory
    let subcategory_id = product_mgmt.create_category(
        manager_id.clone(),
        company_id.clone(),
        "Coffee".to_string(),
        "Coffee products and beans".to_string(),
        Some(category_id.clone()),
        food_category_schema,
    ).expect("Failed to create subcategory");
    
    println!("   📂 Subcategory created: {}", subcategory_id);
    
    // Create a product template
    let mut template_attributes = std::collections::HashMap::new();
    template_attributes.insert("origin".to_string(), "Ethiopia".to_string());
    template_attributes.insert("weight".to_string(), "1.0".to_string());
    template_attributes.insert("organic".to_string(), "true".to_string());
    template_attributes.insert("grade".to_string(), "A".to_string());
    
    let template_id = product_mgmt.create_product_template(
        owner_id.clone(),
        company_id.clone(),
        "Premium Coffee Template".to_string(),
        "Template for premium organic coffee products".to_string(),
        subcategory_id.clone(),
        template_attributes,
    ).expect("Failed to create template");
    
    println!("   📋 Product template created: {}", template_id);
    
    // Create products using template
    let mut attribute_overrides = std::collections::HashMap::new();
    attribute_overrides.insert("origin".to_string(), "Colombia".to_string());
    
    let product_id1 = product_mgmt.create_product_from_template(
        manager_id.clone(),
        template_id.clone(),
        "Colombian Premium Coffee".to_string(),
        Some("Single-origin Colombian coffee beans".to_string()),
        attribute_overrides,
    ).expect("Failed to create product from template");
    
    println!("   ☕ Product created from template: {}", product_id1);
    
    // Create another product directly
    let mut product_attributes = std::collections::HashMap::new();
    product_attributes.insert("origin".to_string(), "Brazil".to_string());
    product_attributes.insert("weight".to_string(), "0.5".to_string());
    product_attributes.insert("organic".to_string(), "false".to_string());
    product_attributes.insert("grade".to_string(), "B".to_string());
    
    let product_id2 = product_mgmt.create_product(
        owner_id.clone(),
        company_id.clone(),
        "Brazilian Coffee Blend".to_string(),
        "Medium roast Brazilian coffee blend".to_string(),
        subcategory_id.clone(),
        product_attributes,
    ).expect("Failed to create product");
    
    println!("   ☕ Product created directly: {}", product_id2);
    
    // Create product batches
    let mut quality_metrics = std::collections::HashMap::new();
    quality_metrics.insert("acidity".to_string(), "4.2".to_string());
    quality_metrics.insert("body".to_string(), "medium".to_string());
    quality_metrics.insert("aroma".to_string(), "strong".to_string());
    
    let batch_id1 = product_mgmt.create_product_batch(
        manager_id.clone(),
        product_id1.clone(),
        "COL-2024-001".to_string(),
        1000, // quantity
        1640995200, // manufacturing date
        Some(1672531200), // expiry date (1 year later)
        quality_metrics.clone(),
    ).expect("Failed to create batch");
    
    println!("   📦 Product batch created: {}", batch_id1);
    
    // Update batch status
    product_mgmt.update_batch_status(
        manager_id.clone(),
        batch_id1.clone(),
        BatchStatus::QualityCheck,
    ).expect("Failed to update batch status");
    
    println!("   ✅ Batch status updated to QualityCheck");
    
    // Update product information
    let mut updated_attributes = std::collections::HashMap::new();
    updated_attributes.insert("origin".to_string(), "Colombia - Huila Region".to_string());
    updated_attributes.insert("weight".to_string(), "1.0".to_string());
    updated_attributes.insert("organic".to_string(), "true".to_string());
    updated_attributes.insert("grade".to_string(), "A".to_string());
    
    product_mgmt.update_product(
        owner_id.clone(),
        product_id1.clone(),
        None,
        Some("Single-origin Colombian coffee from Huila region".to_string()),
        Some(updated_attributes),
    ).expect("Failed to update product");
    
    println!("   🔄 Product updated with detailed origin information");
    
    // Search products
    let search_filter = ProductSearchFilter {
        company_id: Some(company_id.clone()),
        category_id: Some(subcategory_id.clone()),
        name_contains: Some("Coffee".to_string()),
        attributes: {
            let mut attrs = std::collections::HashMap::new();
            attrs.insert("organic".to_string(), "true".to_string());
            attrs
        },
        created_after: None,
        created_before: None,
    };
    
    let search_results = product_mgmt.search_products(
        owner_id.clone(),
        company_id.clone(),
        search_filter,
    ).expect("Failed to search products");
    
    println!("   🔍 Search results (organic coffee): {} products found", search_results.len());
    
    // Get product information
    if let Some(product) = product_mgmt.get_product(&product_id1) {
        println!("   📋 Product details: {} - {}", product.name, product.description);
        println!("      Attributes: {} items", product.attributes.len());
        for (key, value) in &product.attributes {
            println!("         {}: {}", key, value);
        }
    }
    
    // Get category information
    if let Some(category) = product_mgmt.get_category(&subcategory_id) {
        println!("   🏷️  Category: {} with {} attribute schemas", category.name, category.attributes_schema.len());
    }
    
    // Get company products
    let company_products = product_mgmt.get_company_products(&company_id);
    println!("   🏢 Company has {} products", company_products.len());
    
    // Get category products
    let category_products = product_mgmt.get_category_products(&subcategory_id);
    println!("   📂 Coffee category has {} products", category_products.len());
    
    // Get statistics
    let stats = product_mgmt.get_product_stats(&company_id);
    println!("   📊 Product Statistics:");
    println!("      - Total Products: {}", stats.total_products);
    println!("      - Total Categories: {}", stats.total_categories);
    println!("      - Categories Used: {}", stats.categories_used);
    println!("      - Total Batches: {}", stats.total_batches);
    println!("      - Total Templates: {}", stats.total_templates);
    
    // Test validation error
    let mut invalid_attributes = std::collections::HashMap::new();
    invalid_attributes.insert("origin".to_string(), "X".to_string()); // Too short
    invalid_attributes.insert("weight".to_string(), "150.0".to_string()); // Too heavy
    
    match product_mgmt.create_product(
        owner_id.clone(),
        company_id.clone(),
        "Invalid Product".to_string(),
        "This should fail validation".to_string(),
        subcategory_id.clone(),
        invalid_attributes,
    ) {
        Ok(_) => println!("   ❌ Validation should have failed"),
        Err(ProductManagementError::AttributeValidationError(msg)) => {
            println!("   ✅ Validation correctly failed: {}", msg);
        },
        Err(e) => println!("   ❓ Unexpected error: {:?}", e),
    }
    
    println!("   ✅ Product Management pallet working correctly!");
}

fn demo_supply_chain_tracking() {
    println!("\n🚚 Demonstrating Supply Chain Tracking Pallet:");
    
    let mut tracking_system = SupplyChainTracking::new();
    let company_id = "company_1".to_string();
    let owner_id = "user_1".to_string();
    let warehouse_id = "user_3".to_string();
    let transport_id = "user_4".to_string();
    
    // Set up roles
    tracking_system.set_owner_role(company_id.clone(), owner_id.clone());
    tracking_system.assign_role(company_id.clone(), warehouse_id.clone(), UserRole::Warehouse, owner_id.clone())
        .expect("Failed to assign warehouse role");
    tracking_system.assign_role(company_id.clone(), transport_id.clone(), UserRole::Transport, owner_id.clone())
        .expect("Failed to assign transport role");
    
    // Register locations
    let factory_location_id = tracking_system.register_location(
        owner_id.clone(),
        company_id.clone(),
        "Coffee Processing Factory".to_string(),
        "123 Industrial Ave, Medellín, Colombia".to_string(),
        Some(Coordinates { latitude: 6.2442, longitude: -75.5812 }),
        LocationType::Factory,
        "America/Bogota".to_string(),
        Some(ContactInfo {
            phone: Some("+57-4-123-4567".to_string()),
            email: Some("factory@coffee.co".to_string()),
            contact_person: Some("Carlos Rodriguez".to_string()),
        }),
    ).expect("Failed to register factory location");
    
    println!("   🏭 Factory location registered: {}", factory_location_id);
    
    let warehouse_location_id = tracking_system.register_location(
        owner_id.clone(),
        company_id.clone(),
        "Distribution Warehouse".to_string(),
        "456 Logistics Blvd, Bogotá, Colombia".to_string(),
        Some(Coordinates { latitude: 4.6097, longitude: -74.0817 }),
        LocationType::Warehouse,
        "America/Bogota".to_string(),
        Some(ContactInfo {
            phone: Some("+57-1-789-0123".to_string()),
            email: Some("warehouse@coffee.co".to_string()),
            contact_person: Some("Maria Santos".to_string()),
        }),
    ).expect("Failed to register warehouse location");
    
    println!("   📦 Warehouse location registered: {}", warehouse_location_id);
    
    let customer_location_id = tracking_system.register_location(
        owner_id.clone(),
        company_id.clone(),
        "Premium Coffee Store".to_string(),
        "789 Main St, Miami, FL, USA".to_string(),
        Some(Coordinates { latitude: 25.7617, longitude: -80.1918 }),
        LocationType::CustomerLocation,
        "America/New_York".to_string(),
        Some(ContactInfo {
            phone: Some("+1-305-555-0123".to_string()),
            email: Some("orders@premiumcoffee.com".to_string()),
            contact_person: Some("John Smith".to_string()),
        }),
    ).expect("Failed to register customer location");
    
    println!("   🏪 Customer location registered: {}", customer_location_id);
    
    // Create tracking entries for a product journey
    let product_id = "prod_1".to_string();
    
    // Step 1: Production started
    let mut production_metadata = std::collections::HashMap::new();
    production_metadata.insert("batch_size".to_string(), "1000kg".to_string());
    production_metadata.insert("roast_level".to_string(), "medium".to_string());
    
    let entry1_id = tracking_system.create_tracking_entry(
        owner_id.clone(),
        company_id.clone(),
        product_id.clone(),
        TrackingStatus::InProduction,
        factory_location_id.clone(),
        Some("Coffee roasting process initiated".to_string()),
        production_metadata,
        Some(EnvironmentalData {
            temperature: Some(220.0), // Roasting temperature
            humidity: Some(45.0),
            pressure: Some(1013.25),
            vibration: None,
            light_exposure: None,
            recorded_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            sensor_id: Some("TEMP_001".to_string()),
        }),
    ).expect("Failed to create production entry");
    
    println!("   ☕ Production tracking entry created: {}", entry1_id);
    
    // Step 2: Quality check
    let entry2_id = tracking_system.create_tracking_entry(
        warehouse_id.clone(),
        company_id.clone(),
        product_id.clone(),
        TrackingStatus::QualityCheck,
        factory_location_id.clone(),
        Some("Quality control inspection completed".to_string()),
        std::collections::HashMap::new(),
        Some(EnvironmentalData {
            temperature: Some(22.0), // Room temperature
            humidity: Some(60.0),
            pressure: Some(1013.25),
            vibration: None,
            light_exposure: Some(300.0),
            recorded_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            sensor_id: Some("ENV_001".to_string()),
        }),
    ).expect("Failed to create quality check entry");
    
    println!("   ✅ Quality check tracking entry created: {}", entry2_id);
    
    // Add quality certificate document
    let doc_id = tracking_system.add_document_to_entry(
        warehouse_id.clone(),
        company_id.clone(),
        entry2_id.clone(),
        supply_chain_tracking::DocumentType::Certificate,
        "Quality Control Certificate".to_string(),
        "sha256:abc123def456...".to_string(), // Document hash
    ).expect("Failed to add document");
    
    println!("   📄 Quality certificate added: {}", doc_id);
    
    // Step 3: Ready to ship
    let entry3_id = tracking_system.create_tracking_entry(
        warehouse_id.clone(),
        company_id.clone(),
        product_id.clone(),
        TrackingStatus::ReadyToShip,
        warehouse_location_id.clone(),
        Some("Product packaged and ready for shipment".to_string()),
        std::collections::HashMap::new(),
        None,
    ).expect("Failed to create ready to ship entry");
    
    println!("   📋 Ready to ship tracking entry created: {}", entry3_id);
    
    // Create shipment
    let shipment_id = tracking_system.create_shipment(
        transport_id.clone(),
        company_id.clone(),
        vec![product_id.clone()],
        warehouse_location_id.clone(),
        customer_location_id.clone(),
        company_id.clone(), // Self-shipping
        Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 7 * 24 * 60 * 60), // 7 days
        Some("TRACK-COL-US-001".to_string()),
        Some("Handle with care - premium coffee product".to_string()),
    ).expect("Failed to create shipment");
    
    println!("   🚛 Shipment created: {} (Tracking: TRACK-COL-US-001)", shipment_id);
    
    // Step 4: Delivered
    let mut delivery_metadata = std::collections::HashMap::new();
    delivery_metadata.insert("recipient".to_string(), "John Smith".to_string());
    delivery_metadata.insert("delivery_time".to_string(), "14:30 EST".to_string());
    
    let entry4_id = tracking_system.create_tracking_entry(
        transport_id.clone(),
        company_id.clone(),
        product_id.clone(),
        TrackingStatus::Delivered,
        customer_location_id.clone(),
        Some("Successfully delivered to customer".to_string()),
        delivery_metadata,
        None,
    ).expect("Failed to create delivery entry");
    
    println!("   ✅ Delivery tracking entry created: {}", entry4_id);
    
    // Query tracking entries
    let tracking_query = TrackingQuery {
        product_id: Some(product_id.clone()),
        company_id: Some(company_id.clone()),
        status: None,
        location_type: None,
        date_from: None,
        date_to: None,
        operator_id: None,
        shipment_id: None,
    };
    
    let tracking_results = tracking_system.query_tracking_entries(
        owner_id.clone(),
        company_id.clone(),
        tracking_query,
    ).expect("Failed to query tracking entries");
    
    println!("   🔍 Tracking query results: {} entries found", tracking_results.len());
    
    // Display journey details
    if let Some(journey) = tracking_system.get_product_journey(&product_id) {
        println!("   📋 Product Journey Summary:");
        println!("      - Product ID: {}", journey.product_id);
        println!("      - Current Status: {:?}", journey.current_status);
        println!("      - Current Location: {}", journey.current_location.name);
        println!("      - Journey Steps: {}", journey.tracking_entries.len());
        println!("      - Companies Involved: {}", journey.companies_involved.len());
        println!("      - Started: {} (timestamp)", journey.started_at);
        if let Some(delivered_at) = journey.actual_delivery {
            println!("      - Delivered: {} (timestamp)", delivered_at);
            println!("      - Total Journey Time: {} seconds", delivered_at - journey.started_at);
        }
    }
    
    // Show detailed tracking entries
    println!("\n   📊 Detailed Tracking History:");
    for (i, entry) in tracking_results.iter().enumerate() {
        println!("      Step {}: {:?} at {} ({})", 
                 i + 1, entry.status, entry.location.name, entry.timestamp);
        if let Some(ref notes) = entry.notes {
            println!("         Notes: {}", notes);
        }
        if let Some(ref env_data) = entry.environmental_data {
            if let Some(temp) = env_data.temperature {
                println!("         Temperature: {}°C", temp);
            }
            if let Some(humidity) = env_data.humidity {
                println!("         Humidity: {}%", humidity);
            }
        }
        if !entry.documents.is_empty() {
            println!("         Documents: {} attached", entry.documents.len());
        }
    }
    
    // Test different status queries
    let in_transit_query = TrackingQuery {
        product_id: None,
        company_id: Some(company_id.clone()),
        status: Some(TrackingStatus::InTransit),
        location_type: None,
        date_from: None,
        date_to: None,
        operator_id: None,
        shipment_id: None,
    };
    
    let in_transit_results = tracking_system.query_tracking_entries(
        owner_id.clone(),
        company_id.clone(),
        in_transit_query,
    ).expect("Failed to query in-transit entries");
    
    println!("\n   🚛 In-Transit Products: {} found", in_transit_results.len());
    
    // Get tracking statistics
    let stats = tracking_system.get_tracking_stats(&company_id);
    println!("   📈 Tracking Statistics:");
    println!("      - Total Tracking Entries: {}", stats.total_entries);
    println!("      - Active Shipments: {}", stats.active_shipments);
    println!("      - Delivered Products: {}", stats.delivered_products);
    println!("      - In-Transit Products: {}", stats.in_transit_products);
    println!("      - Total Locations: {}", stats.total_locations);
    println!("      - Total Notifications: {}", stats.total_notifications);
    
    // Test invalid status transition (should fail)
    match tracking_system.create_tracking_entry(
        warehouse_id.clone(),
        company_id.clone(),
        product_id.clone(),
        TrackingStatus::InProduction, // Invalid: can't go back to production from delivered
        factory_location_id.clone(),
        Some("Invalid transition test".to_string()),
        std::collections::HashMap::new(),
        None,
    ) {
        Ok(_) => println!("   ❌ Invalid status transition should have failed"),
        Err(SupplyChainTrackingError::InvalidStatusTransition) => {
            println!("   ✅ Invalid status transition correctly rejected");
        },
        Err(e) => println!("   ❓ Unexpected error: {:?}", e),
    }
    
    println!("   ✅ Supply Chain Tracking pallet working correctly!");
}
