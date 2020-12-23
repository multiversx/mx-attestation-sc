#![no_std]
#![allow(clippy::string_lit_as_bytes)]

pub mod user;
mod value_state;

pub use user::User;
pub use value_state::ValueState;

imports!();

#[elrond_wasm_derive::contract(AttestationImpl)]
pub trait Attestation {
	#[init]
	fn init(&self, registration_cost: &BigUint, address: Address, max_nonce_diff: u64) {
		self.set_registration_cost(registration_cost);
		self.set_attestator_state(&address, &ValueState::Approved);

		let mut attestator_list: Vec<Address> = Vec::with_capacity(1);
		attestator_list.push(address);

		self.set_attestator_list(&attestator_list[..]);
		self.set_max_nonce_diff(max_nonce_diff);
	}

	#[endpoint]
	fn version(&self) -> &'static [u8] {
		env!("CARGO_PKG_VERSION").as_bytes()
	}

	#[payable]
	#[endpoint]
	fn register(&self, obfuscated_data: H256, #[payment] payment: BigUint) -> SCResult<()> {
		require!(
			payment == self.get_registration_cost(),
			"should pay exactly the registration cost"
		);

		let mut user_state = self.get_user_or_default(&obfuscated_data);
		require!(
			user_state.value_state != ValueState::Approved,
			"user already registered"
		);

		let caller = self.get_caller();
		if user_state.address == Address::zero() {
			user_state.address = caller;
		} else if user_state.address != caller {
			require!(
				self.get_block_nonce() - user_state.nonce >= self.get_max_nonce_diff(),
				"data already in processing for other user"
			);

			user_state.address = caller;
		}
		if user_state.attester == Address::zero() {
			user_state.attester = self.select_attestator();
		}

		user_state.nonce = self.get_block_nonce();
		if user_state.value_state != ValueState::Pending {
			user_state.value_state = ValueState::Requested;
		}

		self.set_user_state(&obfuscated_data, &user_state);

		Ok(())
	}

	#[endpoint(savePublicInfo)]
	fn save_public_info(&self, obfuscated_data: &H256, public_info: H256) -> SCResult<()> {
		let caller = self.get_caller();
		let attestator_s = self.get_attestator_state(&caller);
		require!(attestator_s.exists(), "caller is not an attestator");

		let mut user_state = self.get_user_or_default(obfuscated_data);

		require!(
			user_state.value_state != ValueState::Approved,
			"user already registered"
		);

		if user_state.address == Address::zero() {
			user_state.attester = caller;
		} else if user_state.attester != caller {
			return sc_error!("not the selected attester");
		}
		let block_nonce = self.get_block_nonce();
		require!(
			block_nonce - user_state.nonce <= self.get_max_nonce_diff(),
			"outside of grace period"
		);

		user_state.public_info = public_info;
		user_state.nonce = block_nonce;
		user_state.value_state = ValueState::Pending;

		self.set_user_state(obfuscated_data, &user_state);

		Ok(())
	}

	#[endpoint]
	fn attest(&self, obfuscated_data: &H256, private_info: BoxedBytes) -> SCResult<()> {
		require!(
			!self.is_empty_user_state(obfuscated_data),
			"no user registered under key"
		);

		let mut user_state = self.get_user_state(obfuscated_data);

		require!(
			user_state.value_state == ValueState::Pending,
			"user already registered"
		);

		let caller = self.get_caller();
		require!(user_state.address == caller, "only user can attest");

		let hashed = self.keccak256(private_info.as_slice());
		require!(
			hashed == user_state.public_info,
			"private/public info mismatch"
		);

		require!(
			self.get_block_nonce() - user_state.nonce <= self.get_max_nonce_diff(),
			"outside of grace period"
		);

		user_state.private_info = private_info;
		user_state.value_state = ValueState::Approved;
		self.set_user_state(obfuscated_data, &user_state);

		Ok(())
	}

	#[endpoint(setRegisterCost)]
	fn set_register_cost(&self, registration_cost: &BigUint) -> SCResult<()> {
		only_owner!(self, "only owner can set registraction cost");

		self.set_registration_cost(registration_cost);
		Ok(())
	}

	#[endpoint(addAttestator)]
	fn add_attestator(&self, address: Address) -> SCResult<()> {
		only_owner!(self, "only owner can add attestator");

		let attestator_s = self.get_attestator_state(&address);
		require!(!attestator_s.exists(), "attestator already exists");

		self.set_attestator_state(&address, &ValueState::Approved);

		let mut attestator_list = self.get_attestator_list();
		attestator_list.push(address);

		self.set_attestator_list(&attestator_list[..]);

		Ok(())
	}

	#[endpoint(removeAttestator)]
	fn remove_attestator(&self, address: &Address) -> SCResult<()> {
		only_owner!(self, "only owner can remove attestator");

		let attestator_s = self.get_attestator_state(address);
		require!(attestator_s.exists(), "attestator does not exists");

		let mut attestator_list = self.get_attestator_list();
		if let Some(pos) = attestator_list.iter().position(|x| x == address) {
			attestator_list.remove(pos);
		}

		require!(!attestator_list.is_empty(), "cannot delete last attestator");

		self.set_attestator_list(&attestator_list[..]);
		self.set_attestator_state(address, &ValueState::None);

		Ok(())
	}

	#[endpoint]
	fn claim(&self) -> SCResult<()> {
		only_owner!(self, "only owner can claim");

		let contract_owner = self.get_owner_address();
		self.send_tx(
			&contract_owner,
			&self.get_sc_balance(),
			b"attestation claim",
		);

		Ok(())
	}

	#[view(getUserData)]
	fn get_user_data(&self, obfuscated_data: &H256) -> SCResult<Box<User>> {
		require!(
			!self.is_empty_user_state(obfuscated_data),
			"no user registered under key"
		);
		let user_state = self.get_user_state(obfuscated_data);
		Ok(user_state)
	}

	#[view(getPublicKey)]
	fn get_public_key(&self, obfuscated_data: &H256) -> SCResult<Address> {
		require!(
			!self.is_empty_user_state(obfuscated_data),
			"no user registered under key"
		);
		let user_state = self.get_user_state(obfuscated_data);
		require!(
			user_state.value_state == ValueState::Approved,
			"userData not yet attested"
		);
		Ok(user_state.address)
	}

	fn select_attestator(&self) -> Address {
		let attestator_list = self.get_attestator_list();
		//TODO add random selection from length of list and the random number
		attestator_list[attestator_list.len() - 1].clone()
	}

	fn default_user(&self) -> Box<User> {
		Box::new(User {
			value_state: ValueState::None,
			public_info: H256::zero(),
			private_info: BoxedBytes::empty(),
			address: Address::zero(),
			attester: Address::zero(),
			nonce: self.get_block_nonce(),
		})
	}

	fn get_user_or_default(&self, obfuscated_data: &H256) -> Box<User> {
		if self.is_empty_user_state(obfuscated_data) {
			self.default_user()
		} else {
			self.get_user_state(obfuscated_data)
		}
	}

	// STORAGE

	#[view(getRegistrationCost)]
	#[storage_get("registration_cost")]
	fn get_registration_cost(&self) -> BigUint;

	#[storage_set("registration_cost")]
	fn set_registration_cost(&self, registration_cost: &BigUint);

	#[view(getMaxNonceDiff)]
	#[storage_get("max_nonce_diff")]
	fn get_max_nonce_diff(&self) -> u64;

	#[storage_set("max_nonce_diff")]
	fn set_max_nonce_diff(&self, max_nonce_diff: u64);

	#[storage_get("attestator_state")]
	fn get_attestator_state(&self, address: &Address) -> ValueState;

	#[storage_set("attestator_state")]
	fn set_attestator_state(&self, address: &Address, value_state: &ValueState);

	#[storage_get("attestator_list")]
	fn get_attestator_list(&self) -> Vec<Address>;

	#[storage_set("attestator_list")]
	fn set_attestator_list(&self, attestator_list: &[Address]);

	#[storage_is_empty("user_state")]
	fn is_empty_user_state(&self, obfuscated_data: &H256) -> bool;

	#[storage_get("user_state")]
	fn get_user_state(&self, obfuscated_data: &H256) -> Box<User>;

	#[storage_set("user_state")]
	fn set_user_state(&self, obfuscated_data: &H256, user: &User);
}
