//! Benchmarking setup for pallet-role-permissions

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as RolePermissions;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_role() {
		let caller: T::AccountId = whitelisted_caller();
		let business_id = 1u32;

		// Setup: Assign owner role to caller first
		let _ = RolePermissions::<T>::assign_role(
			RawOrigin::Root.into(),
			caller.clone(),
			business_id,
			0, // Owner role
		);

		let role_name = vec![0u8; 50];
		let permissions = vec![
			Permission::CreateProduct,
			Permission::ViewProduct,
			Permission::UpdateProduct,
		];

		#[extrinsic_call]
		create_role(
			RawOrigin::Signed(caller.clone()),
			business_id,
			role_name,
			permissions,
		);

		// Verify role was created
		assert!(Roles::<T>::contains_key(5)); // First custom role after 5 system roles
	}

	#[benchmark]
	fn assign_role() {
		let admin: T::AccountId = whitelisted_caller();
		let user: T::AccountId = account("user", 0, 0);
		let business_id = 1u32;

		// Setup: Assign owner role to admin first
		let _ = RolePermissions::<T>::assign_role(
			RawOrigin::Root.into(),
			admin.clone(),
			business_id,
			0, // Owner role
		);

		let role_id = 2u32; // Warehouse role

		#[extrinsic_call]
		assign_role(
			RawOrigin::Signed(admin),
			user.clone(),
			business_id,
			role_id,
		);

		// Verify role was assigned
		assert!(UserRoles::<T>::contains_key(&user, business_id));
	}

	#[benchmark]
	fn revoke_role() {
		let admin: T::AccountId = whitelisted_caller();
		let user: T::AccountId = account("user", 0, 0);
		let business_id = 1u32;

		// Setup: Assign roles
		let _ = RolePermissions::<T>::assign_role(
			RawOrigin::Root.into(),
			admin.clone(),
			business_id,
			0, // Owner role
		);

		let _ = RolePermissions::<T>::assign_role(
			RawOrigin::Signed(admin.clone()).into(),
			user.clone(),
			business_id,
			2, // Warehouse role
		);

		#[extrinsic_call]
		revoke_role(RawOrigin::Signed(admin), user.clone(), business_id);

		// Verify role was revoked
		assert!(!UserRoles::<T>::contains_key(&user, business_id));
	}

	#[benchmark]
	fn update_role_permissions() {
		let caller: T::AccountId = whitelisted_caller();
		let business_id = 1u32;

		// Setup: Assign owner role and create custom role
		let _ = RolePermissions::<T>::assign_role(
			RawOrigin::Root.into(),
			caller.clone(),
			business_id,
			0, // Owner role
		);

		let role_name = vec![0u8; 50];
		let initial_permissions = vec![Permission::ViewProduct];

		let _ = RolePermissions::<T>::create_role(
			RawOrigin::Signed(caller.clone()).into(),
			business_id,
			role_name,
			initial_permissions,
		);

		let role_id = 5u32; // First custom role
		let new_permissions = vec![
			Permission::ViewProduct,
			Permission::CreateProduct,
			Permission::UpdateProduct,
		];

		#[extrinsic_call]
		update_role_permissions(
			RawOrigin::Signed(caller),
			role_id,
			new_permissions.clone(),
		);

		// Verify permissions were updated
		let role = Roles::<T>::get(role_id).unwrap();
		assert_eq!(role.permissions.len(), new_permissions.len());
	}

	impl_benchmark_test_suite!(
		RolePermissions,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
