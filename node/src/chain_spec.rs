use polkadot_sdk::*;

use supply_chain_runtime as runtime;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;

/// Specialized `ChainSpec` for the supply chain parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The default parachain ID for development.
pub const PARACHAIN_ID: u32 = 2000;

/// The relay chain that you want to configure this parachain to connect to.
pub const RELAY_CHAIN: &str = "rococo-local";

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	#[serde(alias = "relayChain", alias = "RelayChain")]
	pub relay_chain: String,
	/// The id of the Parachain.
	#[serde(alias = "paraId", alias = "ParaId")]
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Generate the development chain specification.
pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "SUPC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::builder(
		runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions { relay_chain: RELAY_CHAIN.into(), para_id: PARACHAIN_ID },
	)
	.with_name("Supply Chain Development")
	.with_id("supply-chain-dev")
	.with_chain_type(ChainType::Development)
	.with_properties(properties)
	.build()
}

/// Generate the local testnet chain specification.
pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "SUPC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	#[allow(deprecated)]
	ChainSpec::builder(
		runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions { relay_chain: RELAY_CHAIN.into(), para_id: PARACHAIN_ID },
	)
	.with_name("Supply Chain Local Testnet")
	.with_id("supply-chain-local")
	.with_chain_type(ChainType::Local)
	.with_protocol_id("supply-chain-local")
	.with_properties(properties)
	.build()
}

fn testnet_genesis() -> serde_json::Value {
	use runtime::{AccountId, AuraId, RuntimeGenesisConfig};

	// Alice's address
	let alice: AccountId = sp_core::sr25519::Public::from_ss58check("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
		.expect("Alice address should be valid").into();
	let alice_aura: AuraId = sp_core::sr25519::Public::from_ss58check("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
		.expect("Alice address should be valid").into();

	// Bob's address
	let bob: AccountId = sp_core::sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
		.expect("Bob address should be valid").into();

	// Charlie's address
	let charlie: AccountId = sp_core::sr25519::Public::from_ss58check("5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y")
		.expect("Charlie address should be valid").into();

	let initial_authorities: Vec<(AccountId, AuraId)> = vec![(alice.clone(), alice_aura)];
	let endowed_accounts: Vec<AccountId> = vec![alice.clone(), bob, charlie];
	let root_key = alice;

	let genesis = RuntimeGenesisConfig {
		system: Default::default(),
		balances: pallet_balances::GenesisConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1u128 << 60)).collect(),
			dev_accounts: Default::default(),
		},
		parachain_info: staging_parachain_info::GenesisConfig {
			parachain_id: 2000.into(),
			..Default::default()
		},
		collator_selection: pallet_collator_selection::GenesisConfig {
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			candidacy_bond: runtime::EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: pallet_session::GenesisConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						runtime::SessionKeys { aura: x.1.clone() },
					)
				})
				.collect(),
			..Default::default()
		},
		sudo: pallet_sudo::GenesisConfig {
			key: Some(root_key),
		},
		parachain_system: Default::default(),
		aura: Default::default(),
		aura_ext: Default::default(),
		transaction_payment: Default::default(),
		polkadot_xcm: Default::default(),
		role_permissions: Default::default(),
	};

	serde_json::to_value(&genesis).expect("Genesis config serialization should work")
}

/// Generate standalone development configuration (no relay chain)
/// Use this for testing pallets without parachain overhead
pub fn standalone_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "SUPC".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::builder(
		runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		// Use minimal extensions - no relay chain required
		Extensions { relay_chain: "".into(), para_id: 0 },
	)
	.with_name("Supply Chain Standalone")
	.with_id("supply-chain-standalone")
	.with_chain_type(ChainType::Development)
	.with_genesis_config(testnet_genesis())
	.with_properties(properties)
	.build()
}
