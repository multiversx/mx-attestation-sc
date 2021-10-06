use super::ValueState;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, PartialEq, Debug)]
pub struct User<M: ManagedTypeApi> {
	pub value_state: ValueState,
	pub public_info: H256,
	pub private_info: BoxedBytes,
	pub address: ManagedAddress<M>,
	pub _attester: ManagedAddress<M>, // ignored, only kept for backwards compatibility
	pub nonce: u64,
}

// #[cfg(test)]
// mod codec_tests {
// 	use elrond_wasm::types::{BoxedBytes, ManagedAddress, H256};
// 	use elrond_wasm_debug::{TxContext, check_managed_top_decode, check_managed_top_encode, check_managed_top_encode_decode};

// 	use crate::{User, ValueState};

// 	#[test]
// 	fn test_zeros() {
// 		let api = TxContext::dummy();
// 		let user = User {
// 			value_state: ValueState::None,
// 			public_info: H256::zero(),
// 			private_info: BoxedBytes::empty(),
// 			address: ManagedAddress::zero(api.clone()),
// 			_attester: ManagedAddress::zero(api.clone()),
// 			nonce: 0,
// 		};

// 		check_managed_top_encode_decode(
// 			api.clone(),
// 			user,
// 			&b"abc"[..],
// 		);

// 		// let encoded = check_managed_top_encode(api.clone(), &user);
// 		// let decoded = check_managed_top_decode::<User<TxContext>>(api, encoded.as_slice());
// 		// assert_eq!(user, decoded);
// 	}
// }
