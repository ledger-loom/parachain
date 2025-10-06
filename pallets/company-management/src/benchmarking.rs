//! Benchmarking setup for pallet-company-management

#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[allow(unused)]
use crate::Pallet as CompanyManagement;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_company() {
		let caller: T::AccountId = whitelisted_caller();
		let name = vec![0u8; 100];

		#[extrinsic_call]
		create_company(RawOrigin::Signed(caller), name);
	}

	#[benchmark]
	fn invite_member() {
		let owner: T::AccountId = whitelisted_caller();
		let member: T::AccountId = account("member", 0, 0);
		let _ = CompanyManagement::<T>::create_company(
			RawOrigin::Signed(owner.clone()).into(),
			vec![0u8; 100],
		);

		#[extrinsic_call]
		invite_member(RawOrigin::Signed(owner), 0, member, MemberRole::Manager);
	}

	impl_benchmark_test_suite!(CompanyManagement, crate::mock::new_test_ext(), crate::mock::Test);
}
