//! # Encrypted Data Pallet
//!
//! A FRAME pallet for storing encrypted data on-chain with role-based access control.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Storing encrypted data on-chain
//! - Managing encryption metadata (IV, algorithm, key_id)
//! - Data integrity verification via hashes
//! - Helper functions for encryption operations
//!
//! ## Usage
//!
//! Data should be encrypted OFF-CHAIN before calling extrinsics.
//! This pallet only stores ciphertext and metadata.

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

		/// Maximum length of encrypted data
		#[pallet::constant]
		type MaxEncryptedDataLength: Get<u32>;

		/// Maximum length of key identifier
		#[pallet::constant]
		type MaxKeyIdLength: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Encryption algorithm type
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum EncryptionAlgorithm {
		/// ECIES with P-256 curve
		ECIES,
		/// AES-256-GCM
		AES256GCM,
		/// ChaCha20-Poly1305
		ChaCha20Poly1305,
	}

	/// Encryption metadata
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct EncryptionMetadata<T: Config> {
		/// Encryption algorithm used
		pub algorithm: EncryptionAlgorithm,
		/// Key identifier (ephemeral public key for ECIES, key ID for symmetric)
		pub key_id: BoundedVec<u8, T::MaxKeyIdLength>,
		/// Initialization vector (IV) - 12 bytes for GCM
		pub iv: Option<[u8; 12]>,
		/// Salt used for key derivation (if applicable)
		pub salt: Option<[u8; 32]>,
	}

	/// Encrypted data entry with metadata
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct EncryptedDataEntry<T: Config> {
		/// Encrypted data (ciphertext)
		pub ciphertext: BoundedVec<u8, T::MaxEncryptedDataLength>,
		/// Hash of plaintext for integrity verification
		pub plaintext_hash: [u8; 32],
		/// Encryption metadata
		pub metadata: EncryptionMetadata<T>,
		/// Owner of the data
		pub owner: T::AccountId,
		/// Timestamp of encryption
		pub created_at: BlockNumberFor<T>,
	}

	/// Storage: Encrypted data indexed by hash
	#[pallet::storage]
	#[pallet::getter(fn encrypted_data)]
	pub type EncryptedData<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], EncryptedDataEntry<T>>;

	/// Storage: Data by owner
	#[pallet::storage]
	#[pallet::getter(fn owner_data)]
	pub type OwnerData<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, T::AccountId,
		Blake2_128Concat, [u8; 32],
		(),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Encrypted data stored
		DataStored {
			data_hash: [u8; 32],
			owner: T::AccountId,
			algorithm: EncryptionAlgorithm,
		},
		/// Encrypted data retrieved
		DataRetrieved {
			data_hash: [u8; 32],
			requester: T::AccountId,
		},
		/// Data deleted
		DataDeleted {
			data_hash: [u8; 32],
			owner: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Data not found
		DataNotFound,
		/// Not authorized to access this data
		NotAuthorized,
		/// Encrypted data too long
		EncryptedDataTooLong,
		/// Key ID too long
		KeyIdTooLong,
		/// Data already exists
		DataAlreadyExists,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Store encrypted data on-chain
		///
		/// Data must be encrypted OFF-CHAIN before calling this function.
		/// This function only stores the ciphertext and metadata.
		///
		/// Parameters:
		/// - ciphertext: Encrypted data bytes
		/// - plaintext_hash: SHA-256 hash of original plaintext (for verification)
		/// - metadata: Encryption metadata (algorithm, IV, key_id, salt)
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn store_encrypted_data(
			origin: OriginFor<T>,
			ciphertext: Vec<u8>,
			plaintext_hash: [u8; 32],
			metadata: EncryptionMetadata<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate ciphertext size
			let bounded_ciphertext: BoundedVec<u8, T::MaxEncryptedDataLength> =
				ciphertext.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			// Compute data hash (used as storage key)
			let data_hash = sp_io::hashing::blake2_256(&plaintext_hash);

			// Ensure data doesn't already exist
			ensure!(
				!EncryptedData::<T>::contains_key(&data_hash),
				Error::<T>::DataAlreadyExists
			);

			let now = frame_system::Pallet::<T>::block_number();

			// Create encrypted data entry
			let entry = EncryptedDataEntry {
				ciphertext: bounded_ciphertext,
				plaintext_hash,
				metadata: metadata.clone(),
				owner: who.clone(),
				created_at: now,
			};

			// Store encrypted data
			EncryptedData::<T>::insert(&data_hash, entry);

			// Index by owner
			OwnerData::<T>::insert(&who, &data_hash, ());

			// Emit event
			Self::deposit_event(Event::DataStored {
				data_hash,
				owner: who,
				algorithm: metadata.algorithm,
			});

			Ok(())
		}

		/// Retrieve encrypted data
		#[pallet::call_index(1)]
		#[pallet::weight(5_000)]
		pub fn get_encrypted_data(
			origin: OriginFor<T>,
			data_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure data exists
			ensure!(
				EncryptedData::<T>::contains_key(&data_hash),
				Error::<T>::DataNotFound
			);

			// Emit event (actual data retrieval via RPC)
			Self::deposit_event(Event::DataRetrieved {
				data_hash,
				requester: who,
			});

			Ok(())
		}

		/// Delete encrypted data (owner only)
		#[pallet::call_index(2)]
		#[pallet::weight(5_000)]
		pub fn delete_encrypted_data(
			origin: OriginFor<T>,
			data_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get data and verify ownership
			let entry = EncryptedData::<T>::get(&data_hash)
				.ok_or(Error::<T>::DataNotFound)?;

			ensure!(entry.owner == who, Error::<T>::NotAuthorized);

			// Delete data
			EncryptedData::<T>::remove(&data_hash);
			OwnerData::<T>::remove(&who, &data_hash);

			// Emit event
			Self::deposit_event(Event::DataDeleted {
				data_hash,
				owner: who,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Helper: Create ECIES metadata
		pub fn create_ecies_metadata(
			ephemeral_public_key: Vec<u8>,
			iv: [u8; 12],
		) -> Result<EncryptionMetadata<T>, Error<T>> {
			let key_id: BoundedVec<u8, T::MaxKeyIdLength> =
				ephemeral_public_key.try_into().map_err(|_| Error::<T>::KeyIdTooLong)?;

			Ok(EncryptionMetadata {
				algorithm: EncryptionAlgorithm::ECIES,
				key_id,
				iv: Some(iv),
				salt: None,
			})
		}

		/// Helper: Create AES-256-GCM metadata
		pub fn create_aes_metadata(
			key_id: Vec<u8>,
			iv: [u8; 12],
			salt: Option<[u8; 32]>,
		) -> Result<EncryptionMetadata<T>, Error<T>> {
			let bounded_key_id: BoundedVec<u8, T::MaxKeyIdLength> =
				key_id.try_into().map_err(|_| Error::<T>::KeyIdTooLong)?;

			Ok(EncryptionMetadata {
				algorithm: EncryptionAlgorithm::AES256GCM,
				key_id: bounded_key_id,
				iv: Some(iv),
				salt,
			})
		}

		/// Get all data for an owner
		pub fn get_owner_data(owner: &T::AccountId) -> Vec<[u8; 32]> {
			OwnerData::<T>::iter_prefix(owner)
				.map(|(data_hash, _)| data_hash)
				.collect()
		}
	}
}
