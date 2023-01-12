multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, Clone, PartialEq, Debug)]
pub enum ValueState {
	None,
	Requested,
	Pending,
	Approved,
}

impl ValueState {
	pub fn exists(&self) -> bool {
		!matches!(self, ValueState::None)
	}
}
