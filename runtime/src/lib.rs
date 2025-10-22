#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use polkadot_sdk::cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use polkadot_sdk::sp_api::impl_runtime_apis;
use polkadot_sdk::sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use polkadot_sdk::sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};

use polkadot_sdk::sp_std::prelude::*;
#[cfg(feature = "std")]
use polkadot_sdk::sp_version::NativeVersion;
use polkadot_sdk::sp_version::RuntimeVersion;

use polkadot_sdk::frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	parameter_types,
	traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, Everything, Nothing},
	weights::{ConstantMultiplier, Weight},
	PalletId,
};
use polkadot_sdk::frame_system::limits::{BlockLength, BlockWeights};
pub use polkadot_sdk::sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use polkadot_sdk::sp_runtime::{MultiAddress, Perbill, Permill};

#[cfg(any(feature = "std", test))]
pub use polkadot_sdk::sp_runtime::BuildStorage;

use polkadot_sdk::polkadot_runtime_common::SlowAdjustingFeeUpdate;

// XCM Imports
use polkadot_sdk::staging_xcm::latest::prelude::*;
use polkadot_sdk::staging_xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowTopLevelPaidExecutionFrom,
	DenyReserveTransferToRelayChain, DenyThenTry, EnsureXcmOrigin, FixedWeightBounds,
	FrameTransactionalProcessor, FungibleAdapter, IsConcrete, NativeAsset, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	TrailingSetTopicAsId, UsingComponents, WithComputedOrigin, WithUniqueTopic,
};
use polkadot_sdk::staging_xcm_executor::XcmExecutor;

// FRAME
use polkadot_sdk::frame_system::EnsureRoot;
use polkadot_sdk::cumulus_primitives_core::{AggregateMessageOrigin, ParaId};

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = polkadot_sdk::sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	polkadot_sdk::frame_system::CheckNonZeroSender<Runtime>,
	polkadot_sdk::frame_system::CheckSpecVersion<Runtime>,
	polkadot_sdk::frame_system::CheckTxVersion<Runtime>,
	polkadot_sdk::frame_system::CheckGenesis<Runtime>,
	polkadot_sdk::frame_system::CheckEra<Runtime>,
	polkadot_sdk::frame_system::CheckNonce<Runtime>,
	polkadot_sdk::frame_system::CheckWeight<Runtime>,
	polkadot_sdk::pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = polkadot_sdk::frame_executive::Executive<
	Runtime,
	Block,
	polkadot_sdk::frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use polkadot_sdk::sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use polkadot_sdk::sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("supply-chain-parachain"),
	impl_name: create_runtime_str!("supply-chain-parachain"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// The maximum weight per block.
const WEIGHT_REF_TIME_PER_SECOND: u64 = 1_000_000_000_000;

/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
	polkadot_sdk::cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(Weight::from_parts(10_000_000, 0))
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = Weight::from_parts(10_000_000, 0);
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 42;
}

impl polkadot_sdk::frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type AccountId = AccountId;
	type RuntimeCall = RuntimeCall;
	type Nonce = Nonce;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountData = polkadot_sdk::pallet_balances::AccountData<Balance>;
	type Block = Block;
	type Lookup = polkadot_sdk::sp_runtime::traits::AccountIdLookup<AccountId, ()>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = ();
	type Version = Version;
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = polkadot_sdk::cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = polkadot_sdk::frame_support::traits::ConstU32<16>;
	type BlockHashCount = BlockHashCount;
	type ExtensionsWeightInfo = ();
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl polkadot_sdk::pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl polkadot_sdk::pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type DoneSlashHandler = ();
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 1 * MICROUNIT;
}

pub type WeightToFee = ConstantMultiplier<Balance, ConstU128<{ UNIT }>>;

impl polkadot_sdk::pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = polkadot_sdk::pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, ConstU128<{ 1 * MICROUNIT }>>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightInfo = ();
}

impl polkadot_sdk::pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

// Type alias for ConsensusHook for use in runtime APIs
pub type ConsensusHook = polkadot_sdk::cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	{ MILLISECS_PER_BLOCK as u32 },
	{ BLOCK_PROCESSING_VELOCITY },
	{ UNINCLUDED_SEGMENT_CAPACITY },
>;

impl polkadot_sdk::cumulus_pallet_parachain_system::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = polkadot_sdk::staging_parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type ConsensusHook = ConsensusHook;
	type DmpQueue = polkadot_sdk::frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type SelectCore = polkadot_sdk::cumulus_pallet_parachain_system::DefaultCoreSelector<Runtime>;
}

impl polkadot_sdk::staging_parachain_info::Config for Runtime {}

impl polkadot_sdk::cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl polkadot_sdk::pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as polkadot_sdk::frame_system::Config>::AccountId;
	type ValidatorIdOf = polkadot_sdk::pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = polkadot_sdk::pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = polkadot_sdk::pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	type SessionHandler = <SessionKeys as polkadot_sdk::sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
	type DisablingStrategy = ();
}

impl polkadot_sdk::pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = polkadot_sdk::pallet_aura::MinimumPeriodTimesTwo<Self>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

impl polkadot_sdk::pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinEligibleCollators = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	type ValidatorId = <Self as polkadot_sdk::frame_system::Config>::AccountId;
	type ValidatorIdOf = polkadot_sdk::pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
	type KickThreshold = Period;
}

impl polkadot_sdk::pallet_authorship::Config for Runtime {
	type FindAuthor = polkadot_sdk::pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

// Constants for FixedVelocityConsensusHook
pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;
pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 2;

parameter_types! {
	pub ParentOrParentsExecutivePlurality: Location = Location::parent();
	pub RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

pub type Sibling = polkadot_sdk::polkadot_parachain_primitives::primitives::Sibling;

// Converter from ParaId to AggregateMessageOrigin
pub struct ParaIdToSibling;
impl polkadot_sdk::sp_runtime::traits::Convert<ParaId, AggregateMessageOrigin> for ParaIdToSibling {
	fn convert(para_id: ParaId) -> AggregateMessageOrigin {
		AggregateMessageOrigin::Sibling(para_id)
	}
}

impl polkadot_sdk::pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = polkadot_sdk::pallet_message_queue::mock_helpers::NoopMessageProcessor<
		polkadot_sdk::cumulus_primitives_core::AggregateMessageOrigin,
	>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = polkadot_sdk::staging_xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		polkadot_sdk::staging_xcm_executor::XcmExecutor<XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	type QueueChangeHandler = ();
	type QueuePausedQuery = ();
	type HeapSize = polkadot_sdk::sp_core::ConstU32<{ 64 * 1024 }>;
	type MaxStale = polkadot_sdk::sp_core::ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = MessageQueueServiceWeight;
}

impl polkadot_sdk::cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = polkadot_sdk::polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery<ParaId>;
	type MaxInboundSuspended = polkadot_sdk::sp_core::ConstU32<1_000>;
	type XcmpQueue = polkadot_sdk::frame_support::traits::TransformOrigin<
		MessageQueue,
		AggregateMessageOrigin,
		ParaId,
		ParaIdToSibling,
	>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
}

impl polkadot_sdk::cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DmpSink = polkadot_sdk::frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type WeightInfo = ();
}

parameter_types! {
	pub RelayLocation: Location = Location::parent();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = polkadot_sdk::cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get().unwrap_or(Kusama)), Parachain(ParachainInfo::parachain_id().into())].into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<polkadot_sdk::cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub XcmAssetFeesReceiver: Option<AccountId> = Authorship::author();
}

pub struct XcmConfig;
impl polkadot_sdk::staging_xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = NativeAsset;
	type IsTeleporter = (); // Teleporting is disabled.
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader =
		UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ()>;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
	type XcmEventEmitter = ();
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	polkadot_sdk::cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<Location> = Some(Parent.into());
}

impl polkadot_sdk::pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	// ^ Disable dispatchable execute on the XCM pallet.
	// Needs to be `Everything` for local testing.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Nothing;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = polkadot_sdk::pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = polkadot_sdk::pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AuthorizedAliasConsideration = ();
}

impl polkadot_sdk::cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type Barrier = TrailingSetTopicAsId<
	DenyThenTry<
		DenyReserveTransferToRelayChain,
		(
			TakeWeightCredit,
			WithComputedOrigin<
				(
					AllowTopLevelPaidExecutionFrom<Everything>,
					AllowExplicitUnpaidExecutionFrom<Everything>,
					// ^^^ Parent and its exec plurality get free execution
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;

// ===== Supply Chain Pallets Configuration =====

// User Management Pallet Configuration
impl pallet_user_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_user_management::weights::SubstrateWeight<Runtime>;
	type MaxProfileLength = ConstU32<256>;
	type MaxDocuments = ConstU32<10>;
}

// Company Management Pallet Configuration
impl pallet_company_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_company_management::weights::SubstrateWeight<Runtime>;
	type MaxMembers = ConstU32<100>;
	type MaxCompanyNameLength = ConstU32<128>;
	type MaxPendingInvites = ConstU32<50>;
}

// Product Management Pallet Configuration
impl pallet_product_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_product_management::weights::SubstrateWeight<Runtime>;
	type MaxProductNameLength = ConstU32<128>;
	type MaxCategoryLength = ConstU32<64>;
	type MaxAttributes = ConstU32<20>;
	type MaxAttributeKeyLength = ConstU32<64>;
	type MaxAttributeValueLength = ConstU32<256>;
}

// Supply Chain Tracking Pallet Configuration
impl pallet_supply_chain_tracking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_supply_chain_tracking::weights::SubstrateWeight<Runtime>;
	type MaxLocationLength = ConstU32<128>;
	type MaxNotesLength = ConstU32<512>;
	type MaxEvents = ConstU32<100>;
}

// Role & Permissions Pallet Configuration
impl pallet_role_permissions::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_role_permissions::weights::SubstrateWeight<Runtime>;
	type MaxRoleNameLength = ConstU32<64>;
	type MaxPermissions = ConstU32<20>;
}

// External Integrations Pallet Configuration
impl pallet_external_integrations::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_external_integrations::weights::SubstrateWeight<Runtime>;
	type MaxApiKeyLength = ConstU32<64>;
	type MaxApiKeysPerAccount = ConstU32<10>;
	type MaxBatchSize = ConstU32<100>;
	type MaxEmailRecipients = ConstU32<50>;
	type MaxBarcodeLength = ConstU32<128>;
}

// Security Pallet Configuration
impl pallet_security::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_security::weights::SubstrateWeight<Runtime>;
	type MaxEncryptionKeyLength = ConstU32<256>;
	type MaxAuditLogsPerAccount = ConstU32<1000>;
	type MaxBackupSnapshots = ConstU32<10>;
	type MaxMfaDevices = ConstU32<5>;
	type MaxOAuthProviders = ConstU32<10>;
	type SessionTimeout = ConstU32<3600>;
}

// Construct runtime
construct_runtime!(
	pub struct Runtime {
		// System support stuff.
		System: polkadot_sdk::frame_system,
		ParachainSystem: polkadot_sdk::cumulus_pallet_parachain_system,
		Timestamp: polkadot_sdk::pallet_timestamp,
		ParachainInfo: polkadot_sdk::staging_parachain_info,

		// Monetary stuff.
		Balances: polkadot_sdk::pallet_balances,
		TransactionPayment: polkadot_sdk::pallet_transaction_payment,

		// Governance
		Sudo: polkadot_sdk::pallet_sudo,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: polkadot_sdk::pallet_authorship,
		CollatorSelection: polkadot_sdk::pallet_collator_selection,
		Session: polkadot_sdk::pallet_session,
		Aura: polkadot_sdk::pallet_aura,
		AuraExt: polkadot_sdk::cumulus_pallet_aura_ext,

		// XCM helpers.
		XcmpQueue: polkadot_sdk::cumulus_pallet_xcmp_queue,
		PolkadotXcm: polkadot_sdk::pallet_xcm,
		CumulusXcm: polkadot_sdk::cumulus_pallet_xcm,
		DmpQueue: polkadot_sdk::cumulus_pallet_dmp_queue,
		MessageQueue: polkadot_sdk::pallet_message_queue,

		// Supply Chain Pallets
		UserManagement: pallet_user_management,
		CompanyManagement: pallet_company_management,
		ProductManagement: pallet_product_management,
		SupplyChainTracking: pallet_supply_chain_tracking,
		RolePermissions: pallet_role_permissions,
		ExternalIntegrations: pallet_external_integrations,
		Security: pallet_security,
	}
);

#[cfg(feature = "runtime-benchmarks")]
extern crate polkadot_sdk;

#[cfg(feature = "runtime-benchmarks")]
use polkadot_sdk::frame_benchmarking::list_benchmark;

#[cfg(feature = "runtime-benchmarks")]
use polkadot_sdk::frame_support::traits::WhitelistedStorageKeys;
#[cfg(feature = "runtime-benchmarks")]
use polkadot_sdk::sp_storage::TrackedStorageKey;

impl_runtime_apis! {
	impl polkadot_sdk::sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> polkadot_sdk::sp_consensus_aura::SlotDuration {
			polkadot_sdk::sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			polkadot_sdk::pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl polkadot_sdk::sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> polkadot_sdk::sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl polkadot_sdk::sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> polkadot_sdk::sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl polkadot_sdk::sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: polkadot_sdk::sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: polkadot_sdk::sp_inherents::InherentData,
		) -> polkadot_sdk::sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl polkadot_sdk::sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl polkadot_sdk::sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl polkadot_sdk::sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl polkadot_sdk::frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl polkadot_sdk::cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> polkadot_sdk::cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl polkadot_sdk::cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: polkadot_sdk::sp_consensus_aura::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl polkadot_sdk::frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: polkadot_sdk::frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: polkadot_sdk::frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl polkadot_sdk::frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<polkadot_sdk::frame_benchmarking::BenchmarkList>,
			Vec<polkadot_sdk::frame_support::traits::StorageInfo>,
		) {
			use polkadot_sdk::frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use polkadot_sdk::frame_support::traits::StorageInfoTrait;
			use polkadot_sdk::frame_system_benchmarking::Pallet as SystemBench;
			use polkadot_sdk::cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_session, SessionBench::<Runtime>);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_sudo, Sudo);
			list_benchmark!(list, extra, pallet_collator_selection, CollatorSelection);
			list_benchmark!(list, extra, cumulus_pallet_xcmp_queue, XcmpQueue);

			// Supply chain pallets benchmarks
			list_benchmark!(list, extra, pallet_user_management, UserManagement);
			list_benchmark!(list, extra, pallet_company_management, CompanyManagement);
			list_benchmark!(list, extra, pallet_product_management, ProductManagement);
			list_benchmark!(list, extra, pallet_supply_chain_tracking, SupplyChainTracking);
			list_benchmark!(list, extra, pallet_role_permissions, RolePermissions);
			list_benchmark!(list, extra, pallet_external_integrations, ExternalIntegrations);
			list_benchmark!(list, extra, pallet_security, Security);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: polkadot_sdk::frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<polkadot_sdk::frame_benchmarking::BenchmarkBatch>, polkadot_sdk::sp_runtime::RuntimeString> {
			use polkadot_sdk::frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

			use polkadot_sdk::frame_system_benchmarking::Pallet as SystemBench;
			use polkadot_sdk::cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			impl polkadot_sdk::frame_system_benchmarking::Config for Runtime {}
			impl polkadot_sdk::cumulus_pallet_session_benchmarking::Config for Runtime {}

			use polkadot_sdk::frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_sudo, Sudo);
			add_benchmark!(params, batches, pallet_collator_selection, CollatorSelection);
			add_benchmark!(params, batches, cumulus_pallet_xcmp_queue, XcmpQueue);

			// Supply chain pallets benchmarks
			add_benchmark!(params, batches, pallet_user_management, UserManagement);
			add_benchmark!(params, batches, pallet_company_management, CompanyManagement);
			add_benchmark!(params, batches, pallet_product_management, ProductManagement);
			add_benchmark!(params, batches, pallet_supply_chain_tracking, SupplyChainTracking);
			add_benchmark!(params, batches, pallet_role_permissions, RolePermissions);
			add_benchmark!(params, batches, pallet_external_integrations, ExternalIntegrations);
			add_benchmark!(params, batches, pallet_security, Security);

			Ok(batches)
		}
	}
}

polkadot_sdk::cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = polkadot_sdk::cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}