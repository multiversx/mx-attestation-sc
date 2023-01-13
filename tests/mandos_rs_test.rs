use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
	let mut blockchain = ScenarioWorld::new();

	blockchain
		.register_contract("file:output/attestation.wasm", attestation::ContractBuilder);
	blockchain
}

#[test]
fn attestation_main_rs() {
	multiversx_sc_scenario::run_rs("mandos/main.scen.json", world());
}
