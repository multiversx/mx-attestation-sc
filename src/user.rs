use super::ValueState;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, PartialEq, Debug)]
pub struct User {
	pub value_state: ValueState,
	pub public_info: H256,
	pub private_info: BoxedBytes,
	pub address: Address,
	pub _attester: Address, // ignored, only kept for backwards compatibility
	pub nonce: u64,
}

#[cfg(test)]
mod codec_tests {
	use super::*;
	use elrond_wasm::elrond_codec::test_util::{check_top_decode, check_top_encode};

	#[test]
	fn test_zeros() {
		let user = User {
			value_state: ValueState::None,
			public_info: H256::zero(),
			private_info: BoxedBytes::empty(),
			address: Address::zero(),
			_attester: Address::zero(),
			nonce: 0,
		};
		let encoded = check_top_encode(&user);
		let decoded = check_top_decode::<User>(&encoded[..]);
		assert_eq!(user, decoded);
	}
}
