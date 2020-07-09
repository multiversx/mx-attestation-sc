use elrond_wasm::esd_light::*;

imports!();

#[derive(Clone)]
#[derive(PartialEq)]
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

    fn from_u8(v: u8) -> Result<Self, DecodeError> {
        match v {
            0 => Ok(ValueState::None),
            1 => Ok(ValueState::Requested),
            2 => Ok(ValueState::Pending),
            3 => Ok(ValueState::Approved),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}

impl Encode for ValueState {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.to_u8().dep_encode_to(dest)
	}

	#[inline]
	fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) {
        self.to_u8().using_top_encoded(f)
	}
}

impl Decode for ValueState {
    #[inline]
	fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        ValueState::from_u8(u8::top_decode(input)?)
    }
    
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        ValueState::from_u8(u8::dep_decode(input)?)
    }
}

#[derive(Clone)]
pub struct User {
    pub valueState:  ValueState,
    pub publicInfo:  H256,
    pub privateInfo: H256,
    pub address:     Address,
    pub attester:    Address,
    pub nonce:       u64,
}

impl Encode for User {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.valueState.dep_encode_to(dest);
        self.publicInfo.dep_encode_to(dest);
        self.privateInfo.dep_encode_to(dest);
        self.address.dep_encode_to(dest);
        self.attester.dep_encode_to(dest);
        self.nonce.dep_encode_to(dest);
    }
}
impl Decode for User {
    #[inline]
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(User{
            valueState:  ValueState::dep_decode(input)?,
            publicInfo:  H256::dep_decode(input)?,
            privateInfo: H256::dep_decode(input)?,
            address:     Address::dep_decode(input)?,
            attester:    Address::dep_decode(input)?,
            nonce:       u64::dep_decode(input)?,
        })
    }
}