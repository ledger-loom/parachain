use crate as pallet_security;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU32, ConstU64},
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Security: pallet_security,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const MaxEncryptionKeyLength: u32 = 512;
	pub const MaxAuditLogsPerAccount: u32 = 1000;
	pub const MaxBackupSnapshots: u32 = 100;
	pub const MaxMfaDevices: u32 = 10;
	pub const MaxOAuthProviders: u32 = 10;
	pub const SessionTimeout: u64 = 14400; // ~24 hours
}

impl pallet_security::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxEncryptionKeyLength = MaxEncryptionKeyLength;
	type MaxAuditLogsPerAccount = MaxAuditLogsPerAccount;
	type MaxBackupSnapshots = MaxBackupSnapshots;
	type MaxMfaDevices = MaxMfaDevices;
	type MaxOAuthProviders = MaxOAuthProviders;
	type SessionTimeout = SessionTimeout;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
