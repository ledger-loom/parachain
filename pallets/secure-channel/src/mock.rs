use crate as pallet_secure_channel;
use frame::deps::sp_runtime::{traits::IdentityLookup, BuildStorage};
use frame::testing_prelude::*;

type Block = frame_system::mocking::MockBlock<Test>;

frame::runtime::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		SecureChannel: pallet_secure_channel,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
}

impl pallet_secure_channel::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxPublicKeyLength = ConstU32<33>;
	type MaxKeyRotationHistory = ConstU32<100>;
	type MaxSignatureLength = ConstU32<64>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
