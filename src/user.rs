use elrond_wasm::elrond_codec::*;
use super::ValueState;
use elrond_wasm::{H256, Address, Vec};

#[derive(Clone)]
pub struct User {
    pub value_state:  ValueState,
    pub public_info:  H256,
    pub private_info: Vec<u8>,
    pub address:     Address,
    pub attester:    Address,
    pub nonce:       u64,
}

impl NestedEncode for User {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.value_state.dep_encode(dest)?;
        self.public_info.dep_encode(dest)?;
        self.private_info.dep_encode(dest)?;
        self.address.dep_encode(dest)?;
        self.attester.dep_encode(dest)?;
        self.nonce.dep_encode(dest)?;
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        self.value_state.dep_encode_or_exit(dest, c.clone(), exit);
        self.public_info.dep_encode_or_exit(dest, c.clone(), exit);
        self.private_info.dep_encode_or_exit(dest, c.clone(), exit);
        self.address.dep_encode_or_exit(dest, c.clone(), exit);
        self.attester.dep_encode_or_exit(dest, c.clone(), exit);
        self.nonce.dep_encode_or_exit(dest, c.clone(), exit);
    }
}

impl TopEncode for User {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        top_encode_from_nested(self, output)
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(&self, output: O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        top_encode_from_nested_or_exit(self, output, c, exit);
    }
}

impl NestedDecode for User {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(User{
            value_state:  ValueState::dep_decode(input)?,
            public_info:  H256::dep_decode(input)?,
            private_info: Vec::dep_decode(input)?,
            address:     Address::dep_decode(input)?,
            attester:    Address::dep_decode(input)?,
            nonce:       u64::dep_decode(input)?,
        })
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(input: &mut I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        User{
            value_state:  ValueState::dep_decode_or_exit(input, c.clone(), exit),
            public_info:  H256::dep_decode_or_exit(input, c.clone(), exit),
            private_info: Vec::<u8>::dep_decode_or_exit(input, c.clone(), exit),
            address:     Address::dep_decode_or_exit(input, c.clone(), exit),
            attester:    Address::dep_decode_or_exit(input, c.clone(), exit),
            nonce:       u64::dep_decode_or_exit(input, c.clone(), exit),
        }
    }
}

impl TopDecode for User {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        top_decode_from_nested(input)
    }

    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(input: I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        top_decode_from_nested_or_exit(input, c, exit)
    }
}
