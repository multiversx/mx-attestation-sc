#![no_std]

mod user;
mod value_state;

use user::User;
use value_state::ValueState;

imports!();


#[elrond_wasm_derive::contract(AttestationImpl)]
pub trait Attestation {
    #[init]
    fn init(&self, registration_cost: &BigUint, address: &Address, max_nonce_diff: u64) {
        self.set_registration_cost(registration_cost);
        self.set_attestator_state(address, &ValueState::Approved);
        
        let mut attestator_list: Vec<Address> = Vec::new();
        attestator_list.push(address.clone());

        self.set_attestator_list(&attestator_list);
        self.set_max_nonce_diff(max_nonce_diff);
    }

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    #[payable]
    #[endpoint]
    fn register(&self, obfuscated_data: &H256, #[payment] payment: BigUint) -> SCResult<()> {
        if payment != self.get_registraction_cost() {
            return sc_error!("should pay exactly the registration cost");
        }

        let attestator_s = self.get_attestator_state(obfuscated_data);
        if attestator_s.exists() {
            return sc_error!("is not allowed to save under attestator key")
        }

        let mut opt_user_state = self.get_user_state(obfuscated_data);
        if opt_user_state.is_none() {
            opt_user_state = Some(User {
                value_state:  ValueState::None,
                public_info:  H256::zero(),
                private_info: Vec::new(),
                address:     self.get_caller(),
                attester:    Address::zero(),
                nonce:       self.get_block_nonce(),
            });
        }

        if let Some(user_state) = &mut opt_user_state {
            if user_state.value_state == ValueState::Approved {
                return sc_error!("user already registered");
            }

            if user_state.address == Address::zero() {
                user_state.address = self.get_caller();
            } else if user_state.address != self.get_caller() {
                if self.get_block_nonce() - user_state.nonce < self.get_max_nonce_diff() {
                    return sc_error!("data already in processing for other user");
                }
            
                user_state.address = self.get_caller();
            }
            if user_state.attester == Address::zero() {
                user_state.attester = self.select_attestator();
            }
            
            user_state.nonce = self.get_block_nonce();
            if user_state.value_state != ValueState::Pending {
                user_state.value_state = ValueState::Requested;
            }
    
            self.set_user_state(obfuscated_data, Some(user_state.clone()));
    
            return Ok(())
        } else {
            return sc_error!("impossible error")
        }
    }

    #[endpoint(savePublicInfo)]
    fn save_public_info(&self, obfuscated_data: &H256, public_info: &H256) -> SCResult<()> {
        let attestator_s = self.get_attestator_state(&self.get_caller());
        if !attestator_s.exists() {
            return sc_error!("caller is not an attestator");
        }

        let mut opt_user_state = self.get_user_state(obfuscated_data);
        if opt_user_state.is_none() {
            opt_user_state = Some(User {
                value_state:  ValueState::None,
                public_info:  H256::zero(),
                private_info: Vec::new(),
                address:     Address::zero(),
                attester:    Address::zero(),
                nonce:       self.get_block_nonce(),
            });
        }

        if let Some(user_state) = &mut opt_user_state {
            if user_state.value_state == ValueState::Approved {
                return sc_error!("user already registered");
            }
            if user_state.address == Address::zero() {
                user_state.attester = self.get_caller();
            } else if user_state.attester != self.get_caller() {
                return sc_error!("not the selected attester");
            }
            if self.get_block_nonce() - user_state.nonce > self.get_max_nonce_diff() {
                return sc_error!("outside of grace period");
            }
    
            user_state.public_info = public_info.clone();
            user_state.nonce = self.get_block_nonce();
            user_state.value_state = ValueState::Pending;
    
            self.set_user_state(obfuscated_data, Some(user_state.clone()));
            self.save_public_info_event(&user_state.address, obfuscated_data, public_info);
    
            return Ok(())
        } else {
            return sc_error!("impossible error");
        }
    }

    #[endpoint]
    fn attest(&self, obfuscated_data: &H256, private_info: &Vec<u8>) -> SCResult<()> {
        let mut opt_user_state = self.get_user_state(obfuscated_data);

        if let Some(user_state) = &mut opt_user_state {
            if user_state.value_state != ValueState::Pending {
                return sc_error!("user already registered");
            }
            if user_state.address != self.get_caller() {
                return sc_error!("only user can attest");
            }

            let hashed = self.keccak256(private_info);
            if hashed != user_state.public_info {
                return sc_error!("private/public info missmatch");
            }
            if self.get_block_nonce() - user_state.nonce > self.get_max_nonce_diff() {
                return sc_error!("outside of grace period");
            }

            user_state.private_info = private_info.clone();  
            user_state.value_state = ValueState::Approved;
            self.set_user_state(obfuscated_data, Some(user_state.clone()));

            self.attestation_ok_event(&self.get_caller(), obfuscated_data);

            return Ok(())
        } else {
            return sc_error!("there is not registered user under key");
        }         
    }

    #[endpoint(addAttestator)]
    fn add_attestator(&self, address: &Address) -> SCResult<()> {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can add attestator");
        }

        let attestator_s = self.get_attestator_state(address);
        if attestator_s.exists() {
            return sc_error!("attestator already exists");
        }

        let opt_user_state = self.get_user_state(address);
        if opt_user_state.is_some() {
            return sc_error!("user already exists");
        }

        self.set_attestator_state(address, &ValueState::Approved);
        
        let mut attestator_list = self.get_attestator_list();
        attestator_list.push(address.clone());

        self.set_attestator_list(&attestator_list);

        Ok(())
    }

    #[endpoint(setRegisterCost)]
    fn set_register_cost(&self, registration_cost: &BigUint) -> SCResult<()> {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can add attestator");
        }
        
        self.set_registration_cost(registration_cost);
        Ok(())
    }

    #[endpoint(removeAttestator)]
    fn remove_attestator(&self, address: &Address) -> SCResult<()> {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can add attestator");
        }

        let attestator_s = self.get_attestator_state(address);
        if !attestator_s.exists() {
            return sc_error!("attestator does not exists");
        }
        
        let mut attestator_list = self.get_attestator_list();
        if let Some(pos) = attestator_list.iter().position(|x| *x == address.clone()) {
            attestator_list.remove(pos);
        }

        if attestator_list.len() == 0 {
            return sc_error!("cannot delete last attestator");
        }

        self.set_attestator_list(&attestator_list);
        self.set_attestator_state(address, &ValueState::None);

        Ok(())
    }

    #[endpoint]
    fn claim(&self) -> SCResult<()>  {
        let contract_owner = self.get_owner_address();
        if &self.get_caller() != &contract_owner {
            return sc_error!("only owner can claim");
        }

        self.send_tx(&contract_owner, &self.get_sc_balance(), "attestation claim");

        Ok(())
    }

    #[view(getUserData)]
    fn get_user_data(&self, obfuscated_data: &H256) -> SCResult<User> {
        let opt_user_state = self.get_user_state(obfuscated_data);
        if let Some(user_state) = opt_user_state {
           Ok(user_state)
        } else {
           sc_error!("not data for key")
        }
    }

    #[view(getPublicKey)]
    fn get_public_key(&self, obfuscated_data: &H256) -> SCResult<Address> {
        let opt_user_state = self.get_user_state(obfuscated_data);
        if let Some(user_state) = opt_user_state {
           if user_state.value_state == ValueState::Approved {
                return Ok(user_state.address)
           }
           sc_error!("userData not yet attested")
        } else {
           sc_error!("no data for key")
        }
    }

    fn select_attestator(&self) -> Address {
        let attestator_list = self.get_attestator_list();
        //TODO add random selection from length of list and the random number
        return attestator_list[attestator_list.len() - 1].clone()
    }

    // STORAGE

    #[view(getRegistrationCost)]
    #[storage_get("registration_cost")]
    fn get_registraction_cost(&self) -> BigUint;

    #[storage_set("registration_cost")]
    fn set_registration_cost(&self, registration_cost: &BigUint);

    #[view(getMaxNonceDiff)]
    #[storage_get("max_nonce_diff")]
    fn get_max_nonce_diff(&self) -> u64;

    #[storage_set("max_nonce_diff")]
    fn set_max_nonce_diff(&self, max_nonce_diff: u64);

    #[storage_get("attestator")]
    fn get_attestator_state(&self, address: &Address) -> ValueState;

    #[storage_set("attestator")]
    fn set_attestator_state(&self, address: &Address, value_state: &ValueState);

    #[storage_get("attestator_list")]
    fn get_attestator_list(&self) -> Vec<Address>;

    #[storage_set("attestator_list")]
    fn set_attestator_list(&self, attestator_list: &Vec<Address>);

    #[storage_get("user")]
    fn get_user_state(&self, obfuscated_data: &H256) -> Option<User>;

    #[storage_set("user")]
    fn set_user_state(&self, obfuscated_data: &H256, user: Option<User>);

    // events
    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn register_event(&self, user: &Address, obfuscated_data: &H256, attester: &Address);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn save_public_info_event(&self, user: &Address, obfuscated_data: &H256, public_data: &H256);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn attestation_ok_event(&self, user: &Address, obfuscated_data: &H256);
}