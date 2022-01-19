use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
	let mut blockchain = BlockchainMock::new();

	blockchain
		.register_contract_builder("file:output/attestation.wasm", attestation::ContractBuilder);
	blockchain
}

#[test]
fn attestation_main_rs() {
	elrond_wasm_debug::mandos_rs("mandos/main.scen.json", world());
}
