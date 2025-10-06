use polkadot_sdk::*;

use supply_chain_runtime as runtime;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};

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
	.with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
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
	.with_genesis_config_preset_name(sc_chain_spec::LOCAL_TESTNET_RUNTIME_PRESET)
	.with_protocol_id("supply-chain-local")
	.with_properties(properties)
	.build()
}
