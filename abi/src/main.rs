use attestation::*;
use elrond_wasm_debug::*;

fn main() {
	let contract = AttestationImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
