fn main() {
	#[cfg(feature = "std")]
	{
		polkadot_sdk::substrate_build_script_utils::generate_cargo_keys();
		polkadot_sdk::substrate_build_script_utils::rerun_if_git_head_changed();
	}
}
