//! Benchmarking setup for pallet-supply-chain-tracking

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as SupplyChainTracking;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_tracking() {
		let caller: T::AccountId = whitelisted_caller();
		let product_id = 1u32;
		let business_id = 1u32;
		let location = vec![0u8; 100];

		#[extrinsic_call]
		create_tracking(
			RawOrigin::Signed(caller.clone()),
			product_id,
			business_id,
			location,
		);

		assert!(TrackingRecords::<T>::contains_key(product_id));
	}

	#[benchmark]
	fn add_event() {
		let caller: T::AccountId = whitelisted_caller();
		let product_id = 1u32;
		let business_id = 1u32;
		let initial_location = vec![0u8; 100];

		// Setup: create tracking first
		let _ = SupplyChainTracking::<T>::create_tracking(
			RawOrigin::Signed(caller.clone()).into(),
			product_id,
			business_id,
			initial_location,
		);

		let new_location = vec![1u8; 100];
		let notes = vec![0u8; 200];

		#[extrinsic_call]
		add_event(
			RawOrigin::Signed(caller),
			product_id,
			EventType::Shipped,
			new_location,
			notes,
		);

		let record = TrackingRecords::<T>::get(product_id).unwrap();
		assert_eq!(record.events.len(), 1);
	}

	#[benchmark]
	fn update_status() {
		let caller: T::AccountId = whitelisted_caller();
		let product_id = 1u32;
		let business_id = 1u32;
		let location = vec![0u8; 100];

		// Setup: create tracking first
		let _ = SupplyChainTracking::<T>::create_tracking(
			RawOrigin::Signed(caller.clone()).into(),
			product_id,
			business_id,
			location,
		);

		#[extrinsic_call]
		update_status(
			RawOrigin::Signed(caller),
			product_id,
			TrackingStatus::InProgress,
		);

		let record = TrackingRecords::<T>::get(product_id).unwrap();
		assert_eq!(record.current_status, TrackingStatus::InProgress);
	}

	#[benchmark]
	fn update_location() {
		let caller: T::AccountId = whitelisted_caller();
		let product_id = 1u32;
		let business_id = 1u32;
		let initial_location = vec![0u8; 100];

		// Setup: create tracking first
		let _ = SupplyChainTracking::<T>::create_tracking(
			RawOrigin::Signed(caller.clone()).into(),
			product_id,
			business_id,
			initial_location,
		);

		let new_location = vec![1u8; 100];

		#[extrinsic_call]
		update_location(
			RawOrigin::Signed(caller),
			product_id,
			new_location.clone(),
		);

		let record = TrackingRecords::<T>::get(product_id).unwrap();
		assert_eq!(record.current_location.to_vec(), new_location);
	}

	impl_benchmark_test_suite!(SupplyChainTracking, crate::mock::new_test_ext(), crate::mock::Test);
}
