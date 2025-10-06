//! Mock runtime for testing company management pallet

use crate as pallet_company_management;
use frame::deps::sp_runtime::BuildStorage;
use frame::prelude::*;
use frame::testing_prelude::*;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		CompanyManagement: pallet_company_management,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxCompanyNameLength: u32 = 256;
	pub const MaxMembers: u32 = 100;
	pub const MaxPendingInvites: u32 = 50;
}

impl pallet_company_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxCompanyNameLength = MaxCompanyNameLength;
	type MaxMembers = MaxMembers;
	type MaxPendingInvites = MaxPendingInvites;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
