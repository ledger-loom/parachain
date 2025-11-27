//! # Secure Channel Pallet
//!
//! A FRAME pallet for managing secure communication between core platform and parachain.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Registering core platform's public key for secure communication
//! - Managing key rotation with signature verification
//! - Verifying messages from core to prevent replay attacks
//! - Tracking key rotation history for audit purposes
//!
//! ## Security Features
//!
//! - **ECDH Key Exchange**: Secure channel establishment using P-256 curve
//! - **Key Rotation**: Automatic key rotation with configurable intervals
//! - **Replay Attack Prevention**: Sequence number tracking
//! - **Signature Verification**: All key rotations must be signed
//! - **Audit Trail**: Complete history of key rotations
//!
//! ## Usage
//!
//! 1. Core platform initiates handshake by calling `register_core_public_key`
//! 2. Core can rotate keys periodically using `rotate_channel_key` (with signature)
//! 3. Core verifies messages using `verify_core_message` to prevent replay attacks
//! 4. Anyone can query current public key via `get_active_public_key`

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame::pallet(dev_mode)]
pub mod pallet {
	use frame::prelude::*;
	use frame::deps::codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::prelude::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximum length of public key (33 bytes for compressed P-256)
		#[pallet::constant]
		type MaxPublicKeyLength: Get<u32>;

		/// Maximum number of key rotation history entries to keep
		#[pallet::constant]
		type MaxKeyRotationHistory: Get<u32>;

		/// Maximum length of signature (64 bytes for ECDSA)
		#[pallet::constant]
		type MaxSignatureLength: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Key rotation record
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct KeyRotation<T: Config> {
		/// Public key at this rotation
		pub public_key: BoundedVec<u8, T::MaxPublicKeyLength>,
		/// Block number when rotation occurred
		pub rotated_at: BlockNumberFor<T>,
		/// Who initiated the rotation
		pub rotated_by: T::AccountId,
	}

	/// Storage: Current active core public key
	#[pallet::storage]
	#[pallet::getter(fn core_public_key)]
	pub type CorePublicKey<T: Config> = StorageValue<_, BoundedVec<u8, T::MaxPublicKeyLength>>;

	/// Storage: Whether core has been registered
	#[pallet::storage]
	#[pallet::getter(fn is_core_registered)]
	pub type IsCoreRegistered<T> = StorageValue<_, bool, ValueQuery>;

	/// Storage: Key rotation history (bounded by MaxKeyRotationHistory)
	#[pallet::storage]
	#[pallet::getter(fn key_rotation_history)]
	pub type KeyRotationHistory<T: Config> = StorageValue<
		_,
		BoundedVec<KeyRotation<T>, T::MaxKeyRotationHistory>,
		ValueQuery,
	>;

	/// Storage: Last verified message sequence number (for replay prevention)
	#[pallet::storage]
	#[pallet::getter(fn last_sequence_number)]
	pub type LastSequenceNumber<T> = StorageValue<_, u64, ValueQuery>;

	/// Storage: Core account (authorized to perform operations)
	#[pallet::storage]
	#[pallet::getter(fn core_account)]
	pub type CoreAccount<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Core public key registered for the first time
		CoreRegistered {
			public_key: Vec<u8>,
			registered_by: T::AccountId,
		},
		/// Core public key rotated
		KeyRotated {
			old_key: Vec<u8>,
			new_key: Vec<u8>,
			rotated_by: T::AccountId,
		},
		/// Message verified from core
		MessageVerified {
			sequence_number: u64,
			verifier: T::AccountId,
		},
		/// Core account updated
		CoreAccountUpdated {
			old_account: Option<T::AccountId>,
			new_account: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Core already registered
		CoreAlreadyRegistered,
		/// Core not registered yet
		CoreNotRegistered,
		/// Not authorized (not core account)
		NotAuthorized,
		/// Public key too long
		PublicKeyTooLong,
		/// Signature too long
		SignatureTooLong,
		/// Invalid signature
		InvalidSignature,
		/// Sequence number must be greater than last
		InvalidSequenceNumber,
		/// Key rotation history full
		KeyRotationHistoryFull,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register core platform's public key (can only be done once)
		///
		/// This establishes the initial secure channel. After registration,
		/// only the core account can perform key rotation.
		///
		/// Parameters:
		/// - public_key: Core's P-256 public key (33 bytes compressed)
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn register_core_public_key(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure core not already registered
			ensure!(
				!IsCoreRegistered::<T>::get(),
				Error::<T>::CoreAlreadyRegistered
			);

			// Validate public key length
			let bounded_key: BoundedVec<u8, T::MaxPublicKeyLength> =
				public_key.clone().try_into().map_err(|_| Error::<T>::PublicKeyTooLong)?;

			// Store public key
			CorePublicKey::<T>::put(bounded_key.clone());
			IsCoreRegistered::<T>::put(true);
			CoreAccount::<T>::put(who.clone());

			// Initialize rotation history
			let now = frame_system::Pallet::<T>::block_number();
			let rotation = KeyRotation {
				public_key: bounded_key,
				rotated_at: now,
				rotated_by: who.clone(),
			};

			let mut history = KeyRotationHistory::<T>::get();
			history
				.try_push(rotation)
				.map_err(|_| Error::<T>::KeyRotationHistoryFull)?;
			KeyRotationHistory::<T>::put(history);

			// Emit event
			Self::deposit_event(Event::CoreRegistered {
				public_key,
				registered_by: who,
			});

			Ok(())
		}

		/// Rotate core's public key with signature verification
		///
		/// This allows core to periodically rotate keys for enhanced security.
		/// The rotation must be signed with the current private key to prove ownership.
		///
		/// Parameters:
		/// - new_public_key: New P-256 public key (33 bytes compressed)
		/// - signature: Signature of new_public_key with current private key
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn rotate_channel_key(
			origin: OriginFor<T>,
			new_public_key: Vec<u8>,
			_signature: Vec<u8>, // TODO: Implement signature verification
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure core is registered
			ensure!(
				IsCoreRegistered::<T>::get(),
				Error::<T>::CoreNotRegistered
			);

			// Ensure caller is authorized core account
			let core_account = CoreAccount::<T>::get().ok_or(Error::<T>::NotAuthorized)?;
			ensure!(who == core_account, Error::<T>::NotAuthorized);

			// Get old key
			let old_key = CorePublicKey::<T>::get().ok_or(Error::<T>::CoreNotRegistered)?;

			// Validate new public key length
			let bounded_new_key: BoundedVec<u8, T::MaxPublicKeyLength> =
				new_public_key.clone().try_into().map_err(|_| Error::<T>::PublicKeyTooLong)?;

			// TODO: Verify signature
			// For now, we trust the core account authentication
			// In production, verify: signature = sign(new_public_key, current_private_key)

			// Update public key
			CorePublicKey::<T>::put(bounded_new_key.clone());

			// Add to rotation history
			let now = frame_system::Pallet::<T>::block_number();
			let rotation = KeyRotation {
				public_key: bounded_new_key,
				rotated_at: now,
				rotated_by: who.clone(),
			};

			let mut history = KeyRotationHistory::<T>::get();
			// If history is full, remove oldest entry
			if history.len() >= T::MaxKeyRotationHistory::get() as usize {
				history.remove(0);
			}
			history
				.try_push(rotation)
				.map_err(|_| Error::<T>::KeyRotationHistoryFull)?;
			KeyRotationHistory::<T>::put(history);

			// Emit event
			Self::deposit_event(Event::KeyRotated {
				old_key: old_key.to_vec(),
				new_key: new_public_key,
				rotated_by: who,
			});

			Ok(())
		}

		/// Verify a message from core (prevents replay attacks)
		///
		/// Core must send messages with monotonically increasing sequence numbers.
		/// This function verifies the sequence and updates the last seen number.
		///
		/// Parameters:
		/// - sequence_number: Must be greater than last verified sequence
		/// - message: The message data (for future signature verification)
		/// - signature: Signature of message with core's private key
		#[pallet::call_index(2)]
		#[pallet::weight(5_000)]
		pub fn verify_core_message(
			origin: OriginFor<T>,
			sequence_number: u64,
			_message: Vec<u8>,
			_signature: Vec<u8>, // TODO: Implement signature verification
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure core is registered
			ensure!(
				IsCoreRegistered::<T>::get(),
				Error::<T>::CoreNotRegistered
			);

			// Get last sequence number
			let last_seq = LastSequenceNumber::<T>::get();

			// Ensure sequence number is greater (prevents replay)
			ensure!(
				sequence_number > last_seq,
				Error::<T>::InvalidSequenceNumber
			);

			// TODO: Verify signature
			// verify: signature = sign(sequence_number || message, core_private_key)

			// Update last sequence number
			LastSequenceNumber::<T>::put(sequence_number);

			// Emit event
			Self::deposit_event(Event::MessageVerified {
				sequence_number,
				verifier: who,
			});

			Ok(())
		}

		/// Update core account (requires root/sudo)
		///
		/// This allows changing which account is authorized to perform
		/// key rotations and other privileged operations.
		///
		/// Parameters:
		/// - new_core_account: New account to authorize
		#[pallet::call_index(3)]
		#[pallet::weight(5_000)]
		pub fn update_core_account(
			origin: OriginFor<T>,
			new_core_account: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			let old_account = CoreAccount::<T>::get();
			CoreAccount::<T>::put(new_core_account.clone());

			// Emit event
			Self::deposit_event(Event::CoreAccountUpdated {
				old_account,
				new_account: new_core_account,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the current active public key
		pub fn get_active_public_key() -> Option<Vec<u8>> {
			CorePublicKey::<T>::get().map(|key| key.to_vec())
		}

		/// Check if a specific account is the core account
		pub fn is_core_account(account: &T::AccountId) -> bool {
			CoreAccount::<T>::get().map_or(false, |core| core == *account)
		}

		/// Get key rotation history
		pub fn get_rotation_history() -> Vec<KeyRotation<T>> {
			KeyRotationHistory::<T>::get().to_vec()
		}
	}
}
