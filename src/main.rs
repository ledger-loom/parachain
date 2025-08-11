mod types;
mod user_management;
mod company_management;

use types::*;
use user_management::*;
use company_management::*;

fn main() {
    println!("🚀 Supply Chain Parachain Node Starting...");
    println!("📦 Environment: Development");
    println!("🔧 Status: Company Management pallet implemented");
    println!("📋 Next: Implementing Role & Permissions system");
    println!();
    
    // Demo the core data structures
    demo_core_structures();
    
    // Demo user management functionality
    demo_user_management();
    
    // Demo company management functionality
    demo_company_management();
    
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
            document_type: DocumentType::GovernmentId,
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
