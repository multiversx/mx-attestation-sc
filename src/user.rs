use super::ValueState;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, PartialEq, Debug)]
pub struct User<M: ManagedTypeApi> {
	pub value_state: ValueState,
	pub public_info: ManagedByteArray<M, 32>,
	pub private_info: ManagedBuffer<M>,
	pub address: ManagedAddress<M>,
	pub _attester: ManagedAddress<M>, // ignored, only kept for backwards compatibility
	pub nonce: u64,
}
