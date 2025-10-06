//! Benchmarking setup for pallet-product-management

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as ProductManagement;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_product() {
		let caller: T::AccountId = whitelisted_caller();
		let company_id = 1u32;
		let name = vec![0u8; 100];
		let category = vec![0u8; 50];
		let attributes = vec![
			(vec![0u8; 20], vec![0u8; 50]),
			(vec![1u8; 20], vec![1u8; 50]),
		];

		#[extrinsic_call]
		create_product(
			RawOrigin::Signed(caller.clone()),
			company_id,
			name,
			category,
			attributes,
		);

		assert_eq!(NextProductId::<T>::get(), 1);
	}

	#[benchmark]
	fn update_product_status() {
		let caller: T::AccountId = whitelisted_caller();
		let company_id = 1u32;
		let name = vec![0u8; 100];
		let category = vec![0u8; 50];

		// Setup: create product first
		let _ = ProductManagement::<T>::create_product(
			RawOrigin::Signed(caller.clone()).into(),
			company_id,
			name,
			category,
			vec![],
		);

		let product_id = 0u32;

		#[extrinsic_call]
		update_product_status(
			RawOrigin::Signed(caller),
			product_id,
			ProductStatus::Inactive,
		);
	}

	#[benchmark]
	fn add_attribute() {
		let caller: T::AccountId = whitelisted_caller();
		let company_id = 1u32;
		let name = vec![0u8; 100];
		let category = vec![0u8; 50];

		// Setup: create product first
		let _ = ProductManagement::<T>::create_product(
			RawOrigin::Signed(caller.clone()).into(),
			company_id,
			name,
			category,
			vec![],
		);

		let product_id = 0u32;
		let key = vec![0u8; 20];
		let value = vec![0u8; 50];

		#[extrinsic_call]
		add_attribute(RawOrigin::Signed(caller), product_id, key, value);
	}

	#[benchmark]
	fn update_attribute() {
		let caller: T::AccountId = whitelisted_caller();
		let company_id = 1u32;
		let name = vec![0u8; 100];
		let category = vec![0u8; 50];
		let key = vec![0u8; 20];
		let initial_value = vec![0u8; 50];

		// Setup: create product with an attribute
		let _ = ProductManagement::<T>::create_product(
			RawOrigin::Signed(caller.clone()).into(),
			company_id,
			name,
			category,
			vec![(key.clone(), initial_value)],
		);

		let product_id = 0u32;
		let new_value = vec![1u8; 50];

		#[extrinsic_call]
		update_attribute(RawOrigin::Signed(caller), product_id, key, new_value);
	}

	impl_benchmark_test_suite!(ProductManagement, crate::mock::new_test_ext(), crate::mock::Test);
}
