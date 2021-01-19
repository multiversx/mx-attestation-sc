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

	fn can_overwrite_user_data(&self, obfuscated_data: &H256) -> SCResult<()> {
		if !self.is_empty_user_state(obfuscated_data) {
			let existing_user_state = self.get_user_state(obfuscated_data);
			require!(
				existing_user_state.value_state != ValueState::Approved,
				"user already registered"
			);
			require!(
				self.get_block_nonce() - existing_user_state.nonce >= self.get_max_nonce_diff(),
				"data already registered for other user"
			);
		}
		Ok(())
	}

	/// Called by the user.
	/// Overwrites anything previously saved under `obfuscated_data`, if possible.
	#[payable]
	#[endpoint]
	fn register(&self, obfuscated_data: H256, #[payment] payment: BigUint) -> SCResult<()> {
		require!(
			payment == self.get_registration_cost(),
			"should pay the exact registration cost"
		);

		sc_try!(self.can_overwrite_user_data(&obfuscated_data));

		let user_state = Box::new(User {
			value_state: ValueState::Requested,
			public_info: H256::zero(),
			private_info: BoxedBytes::empty(),
			address: self.get_caller(),
			attester: self.select_attestator(),
			nonce: self.get_block_nonce(),
		});
		self.set_user_state(&obfuscated_data, &user_state);

		Ok(())
	}

	/// Called by the user.
	/// `public_info` is currently the hash of the OTP.
	#[endpoint(saveAttestation)]
	fn save_attestation(&self, obfuscated_data: &H256, public_info: H256) -> SCResult<()> {
		require!(
			!self.is_empty_user_state(obfuscated_data),
			"registraction not started for user"
		);

		let mut user_state = self.get_user_state(obfuscated_data);

		require!(
			user_state.value_state == ValueState::Requested,
			"user not in requested state"
		);

		require!(
			user_state.address == self.get_caller(),
			"only user can attest"
		);

		let block_nonce = self.get_block_nonce();
		require!(
			block_nonce - user_state.nonce <= self.get_max_nonce_diff(),
			"registration period expired"
		);

		user_state.public_info = public_info;
		user_state.nonce = block_nonce;
		user_state.value_state = ValueState::Pending;

		self.set_user_state(obfuscated_data, &user_state);

		Ok(())
	}

	/// Called by the attestator.
	/// Needs to be the correct attestator, as assigned by the contract.
	#[endpoint(confirmAttestation)]
	fn confirm_attestation(
		&self,
		obfuscated_data: &H256,
		private_info: BoxedBytes,
	) -> SCResult<()> {
		let caller = self.get_caller();
		let attestator_s = self.get_attestator_state(&caller);
		require!(attestator_s.exists(), "caller is not an attestator");

		require!(
			!self.is_empty_user_state(obfuscated_data),
			"no user registered under key"
		);

		let mut user_state = self.get_user_state(obfuscated_data);

		require!(
			user_state.value_state == ValueState::Pending,
			"must be called after saveAttestation"
		);

		require!(user_state.attester == caller, "not the selected attester");

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
		only_owner!(self, "only owner can set registration cost");

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

	#[view(getUserState)]
	fn get_user_state_endpoint(&self, obfuscated_data: &H256) -> OptionalResult<Box<User>> {
		if !self.is_empty_user_state(obfuscated_data) {
			OptionalResult::Some(self.get_user_state(obfuscated_data))
		} else {
			OptionalResult::None
		}
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
