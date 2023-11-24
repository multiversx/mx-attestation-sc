use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
	let mut blockchain = ScenarioWorld::new();

	blockchain.register_contract("file:output/attestation.wasm", attestation::ContractBuilder);
	blockchain
}

#[test]
fn main_rs() {
	world().run("scenarios/main.scen.json");
}
