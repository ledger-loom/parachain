//! Mock runtime for testing role permissions pallet

use crate as pallet_role_permissions;
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
		RolePermissions: pallet_role_permissions,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

parameter_types! {
	pub const MaxRoleNameLength: u32 = 128;
	pub const MaxPermissions: u32 = 20;
}

impl pallet_role_permissions::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxRoleNameLength = MaxRoleNameLength;
	type MaxPermissions = MaxPermissions;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into();

	// Initialize system roles
	sp_io::TestExternalities::new(ext).execute_with(|| {
		pallet_role_permissions::Pallet::<Test>::initialize_system_roles();
	});

	ext.into()
}
