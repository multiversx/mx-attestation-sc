use elrond_wasm::*;
use elrond_wasm_debug::*;

#[allow(unused)]
fn contract_map() -> ContractMap<TxContext> {
	let mut contract_map = ContractMap::new();
	contract_map.register_contract(
		"file:../output/attestation.wasm",
		Box::new(|context| Box::new(attestation::contract_obj(context))),
	);
	contract_map
}

#[test]
fn attestation_main_rs() {
	elrond_wasm_debug::mandos_rs("mandos/main.scen.json", &contract_map());
}
