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

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    #[payable]
    #[endpoint]
    fn register(&self, obfuscatedData: &H256, #[payment] payment: BigUint) -> Result<(), SCError> {
        if payment != self.getRegistrationCost() {
            return sc_error!("should pay exactly the registration cost");
        }

        let attestatorS = self._get_attestator_state(obfuscatedData);
        if attestatorS.exists() {
            return sc_error!("is not allowed to save under attestator key")
        }

        let mut optUserState = self._get_user_state(obfuscatedData);
        if optUserState.is_none() {
            optUserState = Some(User {
                valueState:  ValueState::None,
                publicInfo:  H256::zero(),
                privateInfo: Vec::new(),
                address:     self.get_caller(),
                attester:    Address::zero(),
                nonce:       self.get_block_nonce(),
            });
        }

        if let Some(userState) = &mut optUserState {
            if userState.valueState == ValueState::Approved {
                return sc_error!("user already registered");
            }

            if userState.address == Address::zero() {
                userState.address = self.get_caller();
            } else if userState.address != self.get_caller() {
                if self.get_block_nonce() - userState.nonce < self.getMaxNonceDiff() {
                    return sc_error!("data already in processing for other user");
                }
            
                userState.address = self.get_caller();
            }
            if userState.attester == Address::zero() {
                userState.attester = self.selectAttestator();
            }
            
            userState.nonce = self.get_block_nonce();
            if userState.valueState != ValueState::Pending {
                userState.valueState = ValueState::Requested;
            }
    
            self._set_user_state(obfuscatedData, Some(userState.clone()));
    
            return Ok(())
        } else {
            return sc_error!("impossible error")
        }
    }

    #[endpoint]
    fn savePublicInfo(&self, obfuscatedData: &H256, publicInfo: &H256) -> Result<(), SCError> {
        let attestatorS = self._get_attestator_state(&self.get_caller());
        if !attestatorS.exists() {
            return sc_error!("caller is not an attestator");
        }

        let mut optUserState = self._get_user_state(obfuscatedData);
        if optUserState.is_none() {
            optUserState = Some(User {
                valueState:  ValueState::None,
                publicInfo:  H256::zero(),
                privateInfo: Vec::new(),
                address:     Address::zero(),
                attester:    Address::zero(),
                nonce:       self.get_block_nonce(),
            });
        }

        if let Some(userState) = &mut optUserState {
            if userState.valueState == ValueState::Approved {
                return sc_error!("user already registered");
            }
            if userState.address == Address::zero() {
                userState.attester = self.get_caller();
            } else if userState.attester != self.get_caller() {
                return sc_error!("not the selected attester");
            }
            if self.get_block_nonce() - userState.nonce > self.getMaxNonceDiff() {
                return sc_error!("outside of grace period");
            }
    
            userState.publicInfo = publicInfo.clone();
            userState.nonce = self.get_block_nonce();
            userState.valueState = ValueState::Pending;
    
            self._set_user_state(obfuscatedData, Some(userState.clone()));
            self.save_public_info_event(&userState.address, obfuscatedData, publicInfo);
    
            return Ok(())
        } else {
            return sc_error!("impossible error");
        }
    }

    #[endpoint]
    fn attest(&self, obfuscatedData: &H256, privateInfo: &Vec<u8>) -> Result<(), SCError> {
        let mut optUserState = self._get_user_state(obfuscatedData);

        if let Some(userState) = &mut optUserState {
            if userState.valueState != ValueState::Pending {
                return sc_error!("user already registered");
            }
            if userState.address != self.get_caller() {
                return sc_error!("only user can attest");
            }

            let hashed = self.keccak256(privateInfo);
            if hashed != userState.publicInfo.as_bytes() {
                return sc_error!("private/public info missmatch");
            }
            if self.get_block_nonce() - userState.nonce > self.getMaxNonceDiff() {
                return sc_error!("outside of grace period");
            }

            userState.privateInfo = privateInfo.clone();  
            userState.valueState = ValueState::Approved;
            self._set_user_state(obfuscatedData, Some(userState.clone()));

            self.attestation_ok_event(&self.get_caller(), obfuscatedData);

            return Ok(())
        } else {
            return sc_error!("there is not registered user under key");
        }         
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

        let optUserState = self._get_user_state(address);
        if optUserState.is_some() {
            return sc_error!("user already exists");
        }

        self._set_attestator_state(address, &ValueState::Approved);
        
        let mut attestatorList = self._get_attestator_list();
        attestatorList.push(address.clone());

        self._set_attestator_list(&attestatorList);

        Ok(())
    }

    #[endpoint]
    fn setRegisterCost(&self, registration_cost: &BigUint) -> Result<(), SCError> {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can add attestator");
        }
        
        self._set_registration_cost(registration_cost);
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

        self.send_tx(&contract_owner, &self.get_sc_balance(), "attestation claim");

        Ok(())
    }

    #[view(getUserData)]
    fn getUserData(&self, obfuscatedData: &H256) -> Result<User, SCError> {
        let optUserState = self._get_user_state(obfuscatedData);
        if let Some(userState) = optUserState {
           Ok(userState)
        } else {
           sc_error!("not data for key")
        }
    }

    #[view(getPublicKey)]
    fn getPublicKey(&self, obfuscatedData: &H256) -> Result<Address, SCError> {
        let optUserState = self._get_user_state(obfuscatedData);
        if let Some(userState) = optUserState {
           if userState.valueState == ValueState::Approved {
                return Ok(userState.address)
           }
           sc_error!("userData not yet attested")
        } else {
           sc_error!("no data for key")
        }
    }

    #[private]
    fn selectAttestator(&self) -> Address {
        let attestatorList = self._get_attestator_list();
        //TODO add random selection from length of list and the random number
        return attestatorList[attestatorList.len() - 1].clone()
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
    fn _set_user_state(&self, obfuscatedData: &H256, user: Option<User>);

    // events
    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn register_event(&self, user: &Address, obfuscatedData: &H256, attester: &Address);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn save_public_info_event(&self, user: &Address, obfuscatedData: &H256, publicData: &H256);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn attestation_ok_event(&self, user: &Address, obfuscatedData: &H256);
}