//! Mock runtime for testing supply chain tracking pallet

use crate as pallet_supply_chain_tracking;
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
		SupplyChainTracking: pallet_supply_chain_tracking,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxLocationLength: u32 = 128;
	pub const MaxNotesLength: u32 = 512;
	pub const MaxEvents: u32 = 100;
}

impl pallet_supply_chain_tracking::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxLocationLength = MaxLocationLength;
	type MaxNotesLength = MaxNotesLength;
	type MaxEvents = MaxEvents;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
