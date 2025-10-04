use cumulus_client_cli::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use supply_chain_parachain::chain_spec;
use sc_cli::{
	ChainSpec, RuntimeVersion, SubstrateCli,
};
use sp_core::hexdisplay::HexDisplay;

mod command;

fn main() -> sc_cli::Result<()> {
	command::run()
}