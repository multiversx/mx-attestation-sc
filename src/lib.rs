#![no_std]
#![allow(clippy::string_lit_as_bytes)]

pub mod user;
mod value_state;

pub use user::User;
pub use value_state::ValueState;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(AttestationImpl)]
pub trait Attestation {
	#[init]
	fn init(&self, registration_cost: &BigUint, address: Address, max_nonce_diff: u64) {
		self.registration_cost().set(registration_cost);
		self.attestator_state(&address).set(&ValueState::Approved);

		let mut attestator_list: Vec<Address> = Vec::with_capacity(1);
		attestator_list.push(address);

		self.attestator_list().set(&attestator_list);
		self.max_nonce_diff().set(&max_nonce_diff);
	}

	#[endpoint]
	fn version(&self) -> &'static [u8] {
		env!("CARGO_PKG_VERSION").as_bytes()
	}

	fn can_overwrite_user_data(&self, obfuscated_data: &H256) -> SCResult<()> {
		if !self.user_state(obfuscated_data).is_empty() {
			let existing_user_state = self.user_state(obfuscated_data).get();
			require!(
				existing_user_state.value_state != ValueState::Approved,
				"user already registered"
			);
			require!(
				self.blockchain().get_block_nonce() - existing_user_state.nonce
					>= self.max_nonce_diff().get(),
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
			payment == self.registration_cost().get(),
			"should pay the exact registration cost"
		);

		sc_try!(self.can_overwrite_user_data(&obfuscated_data));

		let user_state = Box::new(User {
			value_state: ValueState::Requested,
			public_info: H256::zero(),
			private_info: BoxedBytes::empty(),
			address: self.blockchain().get_caller(),
			attester: self.select_attestator(),
			nonce: self.blockchain().get_block_nonce(),
		});
		self.user_state(&obfuscated_data).set(&user_state);

		Ok(())
	}

	/// Called by the user.
	/// `public_info` is currently the hash of the OTP.
	#[endpoint(saveAttestation)]
	fn save_attestation(&self, obfuscated_data: &H256, public_info: H256) -> SCResult<()> {
		require!(
			!self.user_state(obfuscated_data).is_empty(),
			"registraction not started for user"
		);

		let mut user_state = self.user_state(obfuscated_data).get();

		require!(
			user_state.value_state == ValueState::Requested,
			"user not in requested state"
		);

		require!(
			user_state.address == self.blockchain().get_caller(),
			"only user can attest"
		);

		let block_nonce = self.blockchain().get_block_nonce();
		require!(
			block_nonce - user_state.nonce <= self.max_nonce_diff().get(),
			"registration period expired"
		);

		user_state.public_info = public_info;
		user_state.nonce = block_nonce;
		user_state.value_state = ValueState::Pending;

		self.user_state(obfuscated_data).set(&user_state);

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
		let caller = self.blockchain().get_caller();
		let attestator_s = self.attestator_state(&caller).get();
		require!(attestator_s.exists(), "caller is not an attestator");

		require!(
			!self.user_state(obfuscated_data).is_empty(),
			"no user registered under key"
		);

		let mut user_state = self.user_state(obfuscated_data).get();

		require!(
			user_state.value_state == ValueState::Pending,
			"must be called after saveAttestation"
		);

		require!(user_state.attester == caller, "not the selected attester");

		let hashed = self.crypto().keccak256(private_info.as_slice());
		require!(
			hashed == user_state.public_info,
			"private/public info mismatch"
		);

		require!(
			self.blockchain().get_block_nonce() - user_state.nonce <= self.max_nonce_diff().get(),
			"outside of grace period"
		);

		user_state.private_info = private_info;
		user_state.value_state = ValueState::Approved;
		self.user_state(obfuscated_data).set(&user_state);

		Ok(())
	}

	#[endpoint(setRegisterCost)]
	fn set_register_cost(&self, registration_cost: &BigUint) -> SCResult<()> {
		only_owner!(self, "only owner can set registration cost");

		self.registration_cost().set(registration_cost);
		Ok(())
	}

	#[endpoint(addAttestator)]
	fn add_attestator(&self, address: Address) -> SCResult<()> {
		only_owner!(self, "only owner can add attestator");

		let attestator_s = self.attestator_state(&address).get();
		require!(!attestator_s.exists(), "attestator already exists");

		self.attestator_state(&address).set(&ValueState::Approved);

		self.attestator_list().update(|attestator_list| {
			attestator_list.push(address);
		});

		Ok(())
	}

	#[endpoint(removeAttestator)]
	fn remove_attestator(&self, address: &Address) -> SCResult<()> {
		only_owner!(self, "only owner can remove attestator");

		let attestator_s = self.attestator_state(address).get();
		require!(attestator_s.exists(), "attestator does not exists");

		let attestator_list_empty = self.attestator_list().update(|attestator_list| {
			if let Some(pos) = attestator_list.iter().position(|x| x == address) {
				attestator_list.remove(pos);
			}
			attestator_list.is_empty()
		});

		require!(!attestator_list_empty, "cannot delete last attestator");

		self.attestator_state(address).set(&ValueState::None);

		Ok(())
	}

	#[endpoint]
	fn claim(&self) -> SCResult<()> {
		only_owner!(self, "only owner can claim");

		let contract_owner = self.blockchain().get_owner_address();
		self.send().direct_egld(
			&contract_owner,
			&self.blockchain().get_sc_balance(),
			b"attestation claim",
		);

		Ok(())
	}

	#[view(getUserState)]
	fn get_user_state_endpoint(&self, obfuscated_data: &H256) -> OptionalResult<Box<User>> {
		if !self.user_state(obfuscated_data).is_empty() {
			OptionalResult::Some(self.user_state(obfuscated_data).get())
		} else {
			OptionalResult::None
		}
	}

	#[view(getPublicKey)]
	fn get_public_key(&self, obfuscated_data: &H256) -> SCResult<Address> {
		require!(
			!self.user_state(obfuscated_data).is_empty(),
			"no user registered under key"
		);
		let user_state = self.user_state(obfuscated_data).get();
		require!(
			user_state.value_state == ValueState::Approved,
			"userData not yet attested"
		);
		Ok(user_state.address)
	}

	fn select_attestator(&self) -> Address {
		let attestator_list = self.attestator_list().get();
		//TODO add random selection from length of list and the random number
		attestator_list[attestator_list.len() - 1].clone()
	}

	// STORAGE

	#[view(getRegistrationCost)]
	#[storage_mapper("registration_cost")]
	fn registration_cost(&self) -> SingleValueMapper<Self::Storage, BigUint>;

	#[view(getMaxNonceDiff)]
	#[storage_mapper("max_nonce_diff")]
	fn max_nonce_diff(&self) -> SingleValueMapper<Self::Storage, u64>;

	#[storage_mapper("attestator_state")]
	fn attestator_state(&self, address: &Address) -> SingleValueMapper<Self::Storage, ValueState>;

	#[storage_mapper("attestator_list")]
	fn attestator_list(&self) -> SingleValueMapper<Self::Storage, Vec<Address>>;

	#[storage_mapper("user_state")]
	fn user_state(&self, obfuscated_data: &H256) -> SingleValueMapper<Self::Storage, Box<User>>;
}
