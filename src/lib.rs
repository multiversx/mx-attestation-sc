#![no_std]
#![allow(clippy::string_lit_as_bytes)]

pub mod user;
mod value_state;

pub use user::User;
pub use value_state::ValueState;

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait Attestation {
	#[init]
	fn init(
		&self,
		registration_cost: Self::BigUint,
		max_nonce_diff: u64,
		#[var_args] attesters: VarArgs<Address>,
	) -> SCResult<()> {
		require!(!attesters.is_empty(), "Cannot have empty attester list");

		self.registration_cost().set(&registration_cost);
		self.max_nonce_diff().set(&max_nonce_diff);

		// SetMapper does not have a .clear() method, so we do it the ugly way
		// cannot remove during the iteration, as that would destroy the iterator's internal state
		let nr_old_attesters = self.attestator_list().len();
		if nr_old_attesters > 0 {
			let mut old_attesters = Vec::with_capacity(nr_old_attesters);
			for old_attester in self.attestator_list().iter() {
				old_attesters.push(old_attester);
			}

			for old_attester in old_attesters {
				let _ = self.attestator_list().remove(&old_attester);
				self.attestator_state(&old_attester).clear();
			}
		}

		for attester in attesters.into_vec() {
			self.attestator_state(&attester).set(&ValueState::Approved);
			let _ = self.attestator_list().insert(attester);
		}

		Ok(())
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
	#[payable("EGLD")]
	#[endpoint]
	fn register(&self, obfuscated_data: H256, #[payment] payment: Self::BigUint) -> SCResult<()> {
		require!(
			payment == self.registration_cost().get(),
			"should pay the exact registration cost"
		);

		self.can_overwrite_user_data(&obfuscated_data)?;

		let user_state = Box::new(User {
			value_state: ValueState::Requested,
			public_info: H256::zero(),
			private_info: BoxedBytes::empty(),
			address: self.blockchain().get_caller(),
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
			"registration not started for user"
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
	#[endpoint(confirmAttestation)]
	fn confirm_attestation(&self, obfuscated_data: H256, private_info: BoxedBytes) -> SCResult<()> {
		let caller = self.blockchain().get_caller();
		let attestator_s = self.attestator_state(&caller).get();
		require!(attestator_s.exists(), "caller is not an attestator");

		require!(
			!self.user_state(&obfuscated_data).is_empty(),
			"no user registered under key"
		);

		let mut user_state = self.user_state(&obfuscated_data).get();

		require!(
			user_state.value_state == ValueState::Pending,
			"must be called after saveAttestation"
		);
		require!(
			self.attestator_list().contains(&caller),
			"caller is not an attester"
		);

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
		self.user_state(&obfuscated_data).set(&user_state);

		Ok(())
	}

	#[only_owner]
	#[endpoint(setRegisterCost)]
	fn set_register_cost(&self, registration_cost: Self::BigUint) {
		self.registration_cost().set(&registration_cost);
	}

	#[only_owner]
	#[endpoint(addAttestator)]
	fn add_attestator(&self, address: Address) -> SCResult<()> {
		let attestator_s = self.attestator_state(&address).get();
		require!(!attestator_s.exists(), "attestator already exists");

		self.attestator_state(&address).set(&ValueState::Approved);
		let _ = self.attestator_list().insert(address);

		Ok(())
	}

	#[only_owner]
	#[endpoint(removeAttestator)]
	fn remove_attestator(&self, address: Address) -> SCResult<()> {
		let attestator_s = self.attestator_state(&address).get();
		require!(attestator_s.exists(), "attestator does not exist");

		let _ = self.attestator_list().remove(&address);
		self.attestator_state(&address).set(&ValueState::None);

		require!(
			!self.attestator_list().is_empty(),
			"cannot delete last attestator"
		);

		Ok(())
	}

	#[only_owner]
	#[endpoint]
	fn claim(&self) {
		let egld_token_id = TokenIdentifier::egld();
		let contract_owner = self.blockchain().get_owner_address();
		self.send().direct(
			&contract_owner,
			&egld_token_id,
			0,
			&self.blockchain().get_sc_balance(&egld_token_id, 0),
			b"attestation claim",
		);
	}

	#[view(getUserState)]
	fn get_user_state_endpoint(&self, obfuscated_data: H256) -> OptionalResult<Box<User>> {
		if !self.user_state(&obfuscated_data).is_empty() {
			OptionalResult::Some(self.user_state(&obfuscated_data).get())
		} else {
			OptionalResult::None
		}
	}

	#[view(getPublicKey)]
	fn get_public_key(&self, obfuscated_data: H256) -> SCResult<Address> {
		require!(
			!self.user_state(&obfuscated_data).is_empty(),
			"no user registered under key"
		);
		let user_state = self.user_state(&obfuscated_data).get();
		require!(
			user_state.value_state == ValueState::Approved,
			"userData not yet attested"
		);

		Ok(user_state.address)
	}

	// STORAGE

	#[view(getRegistrationCost)]
	#[storage_mapper("registration_cost")]
	fn registration_cost(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

	#[view(getMaxNonceDiff)]
	#[storage_mapper("max_nonce_diff")]
	fn max_nonce_diff(&self) -> SingleValueMapper<Self::Storage, u64>;

	#[storage_mapper("attestator_state")]
	fn attestator_state(&self, address: &Address) -> SingleValueMapper<Self::Storage, ValueState>;

	#[storage_mapper("attestator_list")]
	fn attestator_list(&self) -> SafeSetMapper<Self::Storage, Address>;

	#[storage_mapper("user_state")]
	fn user_state(&self, obfuscated_data: &H256) -> SingleValueMapper<Self::Storage, Box<User>>;
}
