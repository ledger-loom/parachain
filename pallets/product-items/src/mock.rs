//! Mock runtime for testing product items pallet

use crate as pallet_product_items;
use frame::deps::sp_runtime::BuildStorage;
use frame::prelude::*;
use frame::testing_prelude::*;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		ProductItems: pallet_product_items,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxItemNameLength: u32 = 100;
	pub const MaxUnitLength: u32 = 50;
	pub const MaxDescriptionLength: u32 = 500;
}

impl pallet_product_items::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxItemNameLength = MaxItemNameLength;
	type MaxUnitLength = MaxUnitLength;
	type MaxDescriptionLength = MaxDescriptionLength;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
