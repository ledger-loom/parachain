//! Mock runtime for testing product management pallet

use crate as pallet_product_management;
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
		ProductManagement: pallet_product_management,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxProductNameLength: u32 = 256;
	pub const MaxCategoryLength: u32 = 128;
	pub const MaxAttributes: u32 = 50;
	pub const MaxAttributeKeyLength: u32 = 64;
	pub const MaxAttributeValueLength: u32 = 256;
}

impl pallet_product_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxProductNameLength = MaxProductNameLength;
	type MaxCategoryLength = MaxCategoryLength;
	type MaxAttributes = MaxAttributes;
	type MaxAttributeKeyLength = MaxAttributeKeyLength;
	type MaxAttributeValueLength = MaxAttributeValueLength;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
