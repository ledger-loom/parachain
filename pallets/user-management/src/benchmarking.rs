//! Benchmarking setup for pallet-user-management

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as UserManagement;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_user() {
		let caller: T::AccountId = whitelisted_caller();
		let name = vec![0u8; 100];
		let email_hash = [1u8; 32];

		#[extrinsic_call]
		register_user(RawOrigin::Signed(caller.clone()), name, email_hash);

		assert!(Users::<T>::contains_key(&caller));
	}

	#[benchmark]
	fn update_profile() {
		let caller: T::AccountId = whitelisted_caller();
		let name = vec![0u8; 100];
		let email_hash = [1u8; 32];

		// Setup: register user first
		let _ = UserManagement::<T>::register_user(
			RawOrigin::Signed(caller.clone()).into(),
			name.clone(),
			email_hash,
		);

		let new_name = vec![1u8; 100];

		#[extrinsic_call]
		update_profile(RawOrigin::Signed(caller), Some(new_name));
	}

	#[benchmark]
	fn submit_verification() {
		let caller: T::AccountId = whitelisted_caller();
		let name = vec![0u8; 100];
		let email_hash = [1u8; 32];

		// Setup: register user first
		let _ = UserManagement::<T>::register_user(
			RawOrigin::Signed(caller.clone()).into(),
			name,
			email_hash,
		);

		let doc_hashes = vec![[1u8; 32]];

		#[extrinsic_call]
		submit_verification(
			RawOrigin::Signed(caller),
			VerificationType::Identity,
			doc_hashes,
		);
	}

	#[benchmark]
	fn approve_verification() {
		let user: T::AccountId = whitelisted_caller();
		let name = vec![0u8; 100];
		let email_hash = [1u8; 32];

		// Setup: register user and submit verification
		let _ = UserManagement::<T>::register_user(
			RawOrigin::Signed(user.clone()).into(),
			name,
			email_hash,
		);

		let _ = UserManagement::<T>::submit_verification(
			RawOrigin::Signed(user.clone()).into(),
			VerificationType::Identity,
			vec![[1u8; 32]],
		);

		#[extrinsic_call]
		approve_verification(RawOrigin::Root, user);
	}

	#[benchmark]
	fn reject_verification() {
		let user: T::AccountId = whitelisted_caller();
		let name = vec![0u8; 100];
		let email_hash = [1u8; 32];

		// Setup: register user and submit verification
		let _ = UserManagement::<T>::register_user(
			RawOrigin::Signed(user.clone()).into(),
			name,
			email_hash,
		);

		let _ = UserManagement::<T>::submit_verification(
			RawOrigin::Signed(user.clone()).into(),
			VerificationType::Identity,
			vec![[1u8; 32]],
		);

		#[extrinsic_call]
		reject_verification(RawOrigin::Root, user);
	}

	impl_benchmark_test_suite!(UserManagement, crate::mock::new_test_ext(), crate::mock::Test);
}
