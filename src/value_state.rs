use elrond_wasm::elrond_codec::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ValueState {
	None,
	Requested,
	Pending,
	Approved,
}

impl ValueState {
	pub fn exists(&self) -> bool {
		if let ValueState::None = self {
			false
		} else {
			true
		}
	}

	pub fn to_u8(&self) -> u8 {
		match self {
			ValueState::None => 0,
			ValueState::Requested => 1,
			ValueState::Pending => 2,
			ValueState::Approved => 3,
		}
	}
}

impl NestedEncode for ValueState {
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.to_u8().dep_encode(dest)
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.to_u8().dep_encode_or_exit(dest, c, exit);
	}
}

impl TopEncode for ValueState {
	fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
		self.to_u8().top_encode(output)
	}

	fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(
		&self,
		output: O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.to_u8().top_encode_or_exit(output, c, exit);
	}
}

impl NestedDecode for ValueState {
	fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
		match u8::dep_decode(input)? {
			0 => Ok(ValueState::None),
			1 => Ok(ValueState::Requested),
			2 => Ok(ValueState::Pending),
			3 => Ok(ValueState::Approved),
			_ => Err(DecodeError::INVALID_VALUE),
		}
	}

	fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(
		input: &mut I,
		c: ExitCtx,
		exit: fn(ExitCtx, DecodeError) -> !,
	) -> Self {
		match u8::dep_decode_or_exit(input, c.clone(), exit) {
			0 => ValueState::None,
			1 => ValueState::Requested,
			2 => ValueState::Pending,
			3 => ValueState::Approved,
			_ => exit(c, DecodeError::INVALID_VALUE),
		}
	}
}

impl TopDecode for ValueState {
	fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
		match u8::top_decode(input)? {
			0 => Ok(ValueState::None),
			1 => Ok(ValueState::Requested),
			2 => Ok(ValueState::Pending),
			3 => Ok(ValueState::Approved),
			_ => Err(DecodeError::INVALID_VALUE),
		}
	}

	fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(
		input: I,
		c: ExitCtx,
		exit: fn(ExitCtx, DecodeError) -> !,
	) -> Self {
		match u8::top_decode_or_exit(input, c.clone(), exit) {
			0 => ValueState::None,
			1 => ValueState::Requested,
			2 => ValueState::Pending,
			3 => ValueState::Approved,
			_ => exit(c, DecodeError::INVALID_VALUE),
		}
	}
}
