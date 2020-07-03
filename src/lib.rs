#![no_std]
#![no_main]
#![allow(unused_attributes)]
#![allow(non_snake_case)]

pub mod user_state;
use user_state::*;

imports!();


#[elrond_wasm_derive::contract(AttestationImpl)]
pub trait Attestation {
    #[init]
    fn init(&self, registration_cost: &BigUint, address: &Address, maxNonceDiff: u64) -> Result<(), SCError> {
        self._set_registration_cost(registration_cost);
        self._set_attestator_state(address, &ValueState::Approved);
        
        let mut attestatorList: Vec<Address> = Vec::new();
        attestatorList.push(address.clone());

        self._set_attestator_list(&attestatorList);
        self._set_max_nonce_diff(maxNonceDiff);

        Ok(())
    }

    #[payable]
    #[endpoint]
    fn register(&self, obfuscatedData: &H256, #[payment] payment: BigUint) -> Result<(), SCError> {
        if payment != self.getRegistrationCost() {
            return sc_error!("should pay exactly the registration cost");
        }

        let mut optUserState = self._get_user_state(obfuscatedData);
        if optUserState.is_none() {
            optUserState = Some(User {
                valueState:  ValueState::None,
                publicInfo:  H256::zero(),
                privateInfo: H256::zero(),
                address:     self.get_caller(),
                attester:    Address::zero(),
                nonce:       0,
            });
        }

        let mut userState = optUserState.unwrap();
        if userState.valueState == ValueState::Approved {
            return sc_error!("user already registered");
        }

        userState.attester = self.selectAttestator();
        userState.nonce = self.get_block_nonce();
        userState.valueState = ValueState::Requested;

        self._set_user_state(obfuscatedData, &userState);

        Ok(())
    }

    #[endpoint]
    fn savePublicInfo(&self, obfuscatedData: &H256, publicInfo: &H256) -> Result<(), SCError> {
        let attestatorS = self._get_attestator_state(&self.get_caller());
        if !attestatorS.exists() {
            return sc_error!("caller is not an attestator");
        }

        let optUserState = self._get_user_state(obfuscatedData);
        if optUserState.is_none() {
            return sc_error!("there is not registered user under key");
        }

        let mut userState = optUserState.unwrap();
        if userState.valueState == ValueState::Approved {
            return sc_error!("user already registered");
        }

        if userState.attester != self.get_caller() {
            return sc_error!("not the selected attester");
        }

        userState.publicInfo = publicInfo.clone();
        userState.nonce = self.get_block_nonce();
        userState.valueState = ValueState::Pending;

        self._set_user_state(obfuscatedData, &userState);

        Ok(())
    }

    #[endpoint]
    fn attest(&self, obfuscatedData: &H256, privateInfo: &H256) -> Result<(), SCError> {
        let optUserState = self._get_user_state(obfuscatedData);
        if optUserState.is_none() {
            return sc_error!("there is not registered user under key");
        }

        let mut userState = optUserState.unwrap();
        if userState.valueState != ValueState::Pending {
            return sc_error!("user already registered");
        }

        let hashed = self.keccak256(privateInfo.as_bytes());
        if hashed != userState.publicInfo.as_bytes() {
            return sc_error!("private/public info missmatch");
        }

        userState.privateInfo = privateInfo.clone();
        userState.valueState = ValueState::Approved;

        Ok(())
    }

    #[endpoint]
    fn addAttestator(&self, address: &Address) -> Result<(), SCError> {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can add attestator");
        }

        let attestatorS = self._get_attestator_state(address);
        if attestatorS.exists() {
            return sc_error!("attestator already exists");
        }

        self._set_attestator_state(address, &ValueState::Approved);
        
        let mut attestatorList = self._get_attestator_list();
        attestatorList.push(address.clone());

        self._set_attestator_list(&attestatorList);

        Ok(())
    }

    #[endpoint]
    fn removeAttestator(&self, address: &Address) -> Result<(), SCError> {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can add attestator");
        }

        let attestatorS = self._get_attestator_state(address);
        if !attestatorS.exists() {
            return sc_error!("attestator does not exists");
        }
        
        let mut attestatorList = self._get_attestator_list();
        if let Some(pos) = attestatorList.iter().position(|x| *x == address.clone()) {
            attestatorList.remove(pos);
        }

        if attestatorList.len() == 0 {
            return sc_error!("cannot delete last attestator");
        }

        self._set_attestator_list(&attestatorList);
        self._set_attestator_state(address, &ValueState::None);

        Ok(())
    }

    #[endpoint]
    fn claim(&self) -> Result<(), SCError>  {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can claim");
        }

        self.send_tx(&contract_owner, &self.get_sc_balance(), "dns claim");

        Ok(())
    }

    #[private]
    fn selectAttestator(&self) -> Address {
        let attestatorList = self._get_attestator_list();
        //TODO add random selection from length of list and the random number
        return attestatorList[0].clone()
    }

    // STORAGE

    #[private]
    #[storage_get("registration_cost")]
    fn getRegistrationCost(&self) -> BigUint;

    #[private]
    #[storage_set("registration_cost")]
    fn _set_registration_cost(&self, registration_cost: &BigUint);

    #[private]
    #[storage_get("maxNonceDiff")]
    fn getMaxNonceDiff(&self) -> u64;

    #[private]
    #[storage_set("maxNonceDiff")]
    fn _set_max_nonce_diff(&self, maxNonceDiff: u64);

    #[private]
    #[storage_get("attestator")]
    fn _get_attestator_state(&self, address: &Address) -> ValueState;

    #[private]
    #[storage_set("attestator")]
    fn _set_attestator_state(&self, address: &Address, value_state: &ValueState);

    #[private]
    #[storage_get("attestator_list")]
    fn _get_attestator_list(&self) -> Vec<Address>;

    #[private]
    #[storage_set("attestator_list")]
    fn _set_attestator_list(&self, listAttestator: &Vec<Address>);

    #[private]
    #[storage_get("user")]
    fn _get_user_state(&self, obfuscatedData: &H256) -> Option<User>;

    #[private]
    #[storage_set("user")]
    fn _set_user_state(&self, obfuscatedData: &H256, user: &User);
}