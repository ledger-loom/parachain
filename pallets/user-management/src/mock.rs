//! Mock runtime for testing user management pallet

use crate as pallet_user_management;
use frame::deps::sp_runtime::BuildStorage;
use frame::prelude::*;
use frame::runtime::prelude::*;
use frame::testing_prelude::*;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		UserManagement: pallet_user_management,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxProfileLength: u32 = 256;
	pub const MaxDocuments: u32 = 10;
}

impl pallet_user_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxProfileLength = MaxProfileLength;
	type MaxDocuments = MaxDocuments;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
