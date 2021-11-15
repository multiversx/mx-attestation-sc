/*
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
	let mut blockchain = BlockchainMock::new();
	blockchain.set_current_dir_from_workspace("");

	blockchain.register_contract(
		"file:output/attestation.wasm",
		Box::new(|context| Box::new(attestation::contract_obj(context))),
	);
	blockchain
}

#[test]
fn attestation_main_rs() {
	elrond_wasm_debug::mandos_rs("mandos/main.scen.json", world());
}
*/
