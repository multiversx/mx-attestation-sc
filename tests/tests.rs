extern crate attestation;
use attestation::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
	let mut contract_map = ContractMap::new();
	contract_map.register_contract(
		"file:../output/attestation.wasm",
		Box::new(|context| Box::new(AttestationImpl::new(context))),
	);
	contract_map
}

#[test]
fn big_test() {
	parse_execute_mandos("mandos/main.scen.json", &contract_map());
}
