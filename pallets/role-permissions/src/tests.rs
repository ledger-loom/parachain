//! Unit tests for role permissions pallet

use crate::{mock::*, Error, Event, Permission};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

type RolePermissions = crate::Pallet<Test>;

#[test]
fn system_roles_initialized_correctly() {
	new_test_ext().execute_with(|| {
		// Check that 5 system roles were created (Owner, Manager, Warehouse, Transport, Supplier)
		assert_eq!(crate::NextRoleId::<Test>::get(), 5);

		// Verify Owner role (role_id: 0) has all permissions
		let owner_role = crate::Roles::<Test>::get(0).unwrap();
		assert_eq!(owner_role.name.to_vec(), b"Owner".to_vec());
		assert!(owner_role.is_system_role);
		assert_eq!(owner_role.permissions.len(), 10); // All 10 permissions

		// Verify Manager role (role_id: 1)
		let manager_role = crate::Roles::<Test>::get(1).unwrap();
		assert_eq!(manager_role.name.to_vec(), b"Manager".to_vec());
		assert!(manager_role.is_system_role);
		assert_eq!(manager_role.permissions.len(), 9); // All except ManageCompany

		// Verify Warehouse role (role_id: 2)
		let warehouse_role = crate::Roles::<Test>::get(2).unwrap();
		assert_eq!(warehouse_role.name.to_vec(), b"Warehouse".to_vec());
		assert!(warehouse_role.is_system_role);

		// Verify Transport role (role_id: 3)
		let transport_role = crate::Roles::<Test>::get(3).unwrap();
		assert_eq!(transport_role.name.to_vec(), b"Transport".to_vec());
		assert!(transport_role.is_system_role);

		// Verify Supplier role (role_id: 4)
		let supplier_role = crate::Roles::<Test>::get(4).unwrap();
		assert_eq!(supplier_role.name.to_vec(), b"Supplier".to_vec());
		assert!(supplier_role.is_system_role);
	});
}

#[test]
fn create_role_works() {
	new_test_ext().execute_with(|| {
		let owner = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// First, assign owner role to user
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			owner.clone(),
			company_id,
			0 // Owner role
		));

		// Create a custom role
		let role_name = b"CustomRole".to_vec();
		let permissions = vec![Permission::ViewProduct, Permission::CreateProduct];

		assert_ok!(RolePermissions::create_role(
			RuntimeOrigin::signed(owner.clone()),
			company_id,
			role_name.clone(),
			permissions.clone()
		));

		// Verify role was created
		let role_id = 5; // Next role after system roles (0-4)
		let role = crate::Roles::<Test>::get(role_id).unwrap();
		assert_eq!(role.name.to_vec(), role_name);
		assert_eq!(role.company_id, Some(company_id));
		assert!(!role.is_system_role);
		assert_eq!(role.permissions.to_vec(), permissions);

		// Verify event
		System::assert_last_event(
			Event::RoleCreated {
				role_id,
				company_id: Some(company_id),
				name: role_name,
			}
			.into(),
		);
	});
}

#[test]
fn create_role_fails_without_permission() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// Try to create role without ManageRoles permission
		assert_noop!(
			RolePermissions::create_role(
				RuntimeOrigin::signed(user),
				company_id,
				b"TestRole".to_vec(),
				vec![Permission::ViewProduct]
			),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn assign_role_works() {
	new_test_ext().execute_with(|| {
		let admin = AccountId32::from([1u8; 32]);
		let user = AccountId32::from([2u8; 32]);
		let company_id = 1u32;

		// Assign owner role to admin
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			admin.clone(),
			company_id,
			0 // Owner role
		));

		// Admin assigns warehouse role to user
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::signed(admin),
			user.clone(),
			company_id,
			2 // Warehouse role
		));

		// Verify role assignment
		let assigned_role_id = crate::UserRoles::<Test>::get(&user, company_id).unwrap();
		assert_eq!(assigned_role_id, 2);

		// Verify event
		System::assert_last_event(
			Event::RoleAssigned {
				user,
				role_id: 2,
				company_id,
			}
			.into(),
		);
	});
}

#[test]
fn assign_role_fails_if_already_assigned() {
	new_test_ext().execute_with(|| {
		let admin = AccountId32::from([1u8; 32]);
		let user = AccountId32::from([2u8; 32]);
		let company_id = 1u32;

		// Assign owner role to admin
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			admin.clone(),
			company_id,
			0
		));

		// Assign role to user
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::signed(admin.clone()),
			user.clone(),
			company_id,
			2
		));

		// Try to assign another role to same user in same company
		assert_noop!(
			RolePermissions::assign_role(
				RuntimeOrigin::signed(admin),
				user,
				company_id,
				3
			),
			Error::<Test>::RoleAlreadyAssigned
		);
	});
}

#[test]
fn revoke_role_works() {
	new_test_ext().execute_with(|| {
		let admin = AccountId32::from([1u8; 32]);
		let user = AccountId32::from([2u8; 32]);
		let company_id = 1u32;

		// Assign owner role to admin
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			admin.clone(),
			company_id,
			0
		));

		// Assign role to user
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::signed(admin.clone()),
			user.clone(),
			company_id,
			2
		));

		// Revoke role
		assert_ok!(RolePermissions::revoke_role(
			RuntimeOrigin::signed(admin),
			user.clone(),
			company_id
		));

		// Verify role was revoked
		assert!(!crate::UserRoles::<Test>::contains_key(&user, company_id));

		// Verify event
		System::assert_last_event(
			Event::RoleRevoked {
				user,
				company_id,
			}
			.into(),
		);
	});
}

#[test]
fn revoke_role_fails_if_no_role_assigned() {
	new_test_ext().execute_with(|| {
		let admin = AccountId32::from([1u8; 32]);
		let user = AccountId32::from([2u8; 32]);
		let company_id = 1u32;

		// Assign owner role to admin
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			admin.clone(),
			company_id,
			0
		));

		// Try to revoke role from user with no role
		assert_noop!(
			RolePermissions::revoke_role(RuntimeOrigin::signed(admin), user, company_id),
			Error::<Test>::NoRoleAssigned
		);
	});
}

#[test]
fn update_role_permissions_works() {
	new_test_ext().execute_with(|| {
		let owner = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// Assign owner role to user
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			owner.clone(),
			company_id,
			0
		));

		// Create a custom role
		assert_ok!(RolePermissions::create_role(
			RuntimeOrigin::signed(owner.clone()),
			company_id,
			b"TestRole".to_vec(),
			vec![Permission::ViewProduct]
		));

		let role_id = 5;

		// Update permissions
		let new_permissions = vec![Permission::ViewProduct, Permission::CreateProduct];
		assert_ok!(RolePermissions::update_role_permissions(
			RuntimeOrigin::signed(owner),
			role_id,
			new_permissions.clone()
		));

		// Verify permissions were updated
		let role = crate::Roles::<Test>::get(role_id).unwrap();
		assert_eq!(role.permissions.to_vec(), new_permissions);

		// Verify event
		System::assert_last_event(Event::PermissionsUpdated { role_id }.into());
	});
}

#[test]
fn update_role_permissions_fails_for_system_roles() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);

		// Try to update system role permissions
		assert_noop!(
			RolePermissions::update_role_permissions(
				RuntimeOrigin::signed(user),
				0, // Owner role (system role)
				vec![Permission::ViewProduct]
			),
			Error::<Test>::CannotModifySystemRole
		);
	});
}

#[test]
fn check_permission_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// Assign warehouse role to user (has CreateProduct permission)
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			user.clone(),
			company_id,
			2 // Warehouse role
		));

		// Check permission
		assert!(RolePermissions::check_permission(
			&user,
			company_id,
			Permission::CreateProduct
		));
		assert!(RolePermissions::check_permission(
			&user,
			company_id,
			Permission::ViewProduct
		));

		// Check permission user doesn't have
		assert!(!RolePermissions::check_permission(
			&user,
			company_id,
			Permission::ManageCompany
		));
	});
}

#[test]
fn check_permission_fails_for_different_company() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id_1 = 1u32;
		let company_id_2 = 2u32;

		// Assign role to user in company 1
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			user.clone(),
			company_id_1,
			0 // Owner role
		));

		// Check permission in company 1 - should work
		assert!(RolePermissions::check_permission(
			&user,
			company_id_1,
			Permission::ManageCompany
		));

		// Check permission in company 2 - should fail
		assert!(!RolePermissions::check_permission(
			&user,
			company_id_2,
			Permission::ManageCompany
		));
	});
}

#[test]
fn get_user_permissions_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// Assign supplier role to user
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			user.clone(),
			company_id,
			4 // Supplier role
		));

		// Get permissions
		let permissions = RolePermissions::get_user_permissions(&user, company_id).unwrap();
		assert_eq!(permissions.len(), 3);
		assert!(permissions.contains(&Permission::CreateProduct));
		assert!(permissions.contains(&Permission::ViewProduct));
		assert!(permissions.contains(&Permission::ViewReports));
	});
}

#[test]
fn is_system_role_works() {
	new_test_ext().execute_with(|| {
		// Check system roles
		assert!(RolePermissions::is_system_role(0)); // Owner
		assert!(RolePermissions::is_system_role(1)); // Manager
		assert!(RolePermissions::is_system_role(2)); // Warehouse
		assert!(RolePermissions::is_system_role(3)); // Transport
		assert!(RolePermissions::is_system_role(4)); // Supplier

		// Check non-system role
		assert!(!RolePermissions::is_system_role(5));
	});
}

#[test]
fn owner_role_has_all_permissions() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// Assign owner role
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			user.clone(),
			company_id,
			0 // Owner role
		));

		// Check all permissions
		assert!(RolePermissions::check_permission(&user, company_id, Permission::CreateProduct));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::UpdateProduct));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::DeleteProduct));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::ViewProduct));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::ManageUsers));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::ManageRoles));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::ViewReports));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::CreateShipment));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::UpdateShipment));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::ManageCompany));
	});
}

#[test]
fn manager_role_missing_manage_company() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;

		// Assign manager role
		assert_ok!(RolePermissions::assign_role(
			RuntimeOrigin::root(),
			user.clone(),
			company_id,
			1 // Manager role
		));

		// Check has most permissions
		assert!(RolePermissions::check_permission(&user, company_id, Permission::CreateProduct));
		assert!(RolePermissions::check_permission(&user, company_id, Permission::ManageUsers));

		// Check doesn't have ManageCompany
		assert!(!RolePermissions::check_permission(&user, company_id, Permission::ManageCompany));
	});
}
