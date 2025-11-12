//! # Encrypted Data Pallet
//!
//! A FRAME pallet for storing encrypted data on-chain with role-based access control.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Storing encrypted data on-chain
//! - Managing encryption keys per company/role
//! - Role-based access control for viewing encrypted data
//! - Data integrity verification via hashes
//! - Audit logging for data access

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

		/// Maximum number of authorized roles per data entry
		#[pallet::constant]
		type MaxAuthorizedRoles: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Data visibility levels
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VisibilityLevel {
		/// Anyone can access
		Public,
		/// Only company members
		Company,
		/// Only managers and admins
		Management,
		/// Only specific roles
		Restricted,
		/// Only owner
		Private,
	}

	/// Encryption algorithm type
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum EncryptionAlgorithm {
		AES256,
		ChaCha20,
		AES128,
	}

	/// Encryption metadata
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct EncryptionMetadata<T: Config> {
		/// Encryption algorithm used
		pub algorithm: EncryptionAlgorithm,
		/// Key identifier (not the actual key)
		pub key_id: BoundedVec<u8, T::MaxKeyIdLength>,
		/// Salt used for key derivation (if applicable)
		pub salt: Option<[u8; 32]>,
		/// Initialization vector (IV)
		pub iv: Option<[u8; 16]>,
	}

	/// Encrypted data entry
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
		/// Company ID
		pub company_id: u32,
		/// Visibility level
		pub visibility: VisibilityLevel,
		/// Authorized roles (if visibility is Restricted)
		pub authorized_roles: BoundedVec<u32, T::MaxAuthorizedRoles>,
		/// Creation timestamp
		pub created_at: BlockNumberFor<T>,
		/// Last updated timestamp
		pub updated_at: BlockNumberFor<T>,
	}

	/// Company encryption key info (stores metadata, NOT the actual key)
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct CompanyKeyInfo<T: Config> {
		/// Key identifier
		pub key_id: BoundedVec<u8, T::MaxKeyIdLength>,
		/// Public key hash (for verification)
		pub key_hash: [u8; 32],
		/// Algorithm
		pub algorithm: EncryptionAlgorithm,
		/// Is active
		pub is_active: bool,
		/// Created at
		pub created_at: BlockNumberFor<T>,
	}

	/// Storage: Encrypted data entries by ID
	#[pallet::storage]
	#[pallet::getter(fn encrypted_data)]
	pub type EncryptedData<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], EncryptedDataEntry<T>>;

	/// Storage: Company encryption keys
	#[pallet::storage]
	#[pallet::getter(fn company_keys)]
	pub type CompanyKeys<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, u32,  // company_id
		Blake2_128Concat, BoundedVec<u8, T::MaxKeyIdLength>,  // key_id
		CompanyKeyInfo<T>,
	>;

	/// Storage: Active key per company
	#[pallet::storage]
	#[pallet::getter(fn active_company_key)]
	pub type ActiveCompanyKey<T: Config> = StorageMap<_, Blake2_128Concat, u32, BoundedVec<u8, T::MaxKeyIdLength>>;

	/// Storage: Data access log (for audit)
	#[pallet::storage]
	#[pallet::getter(fn access_log)]
	pub type AccessLog<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		([u8; 32], T::AccountId, BlockNumberFor<T>),  // (data_id, accessor, block)
		(),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Encrypted data stored
		EncryptedDataStored {
			data_id: [u8; 32],
			owner: T::AccountId,
			company_id: u32,
			visibility: VisibilityLevel,
		},
		/// Encrypted data updated
		EncryptedDataUpdated {
			data_id: [u8; 32],
			updater: T::AccountId,
		},
		/// Encrypted data accessed
		EncryptedDataAccessed {
			data_id: [u8; 32],
			accessor: T::AccountId,
		},
		/// Company encryption key registered
		CompanyKeyRegistered {
			company_id: u32,
			key_id: Vec<u8>,
		},
		/// Company encryption key rotated
		CompanyKeyRotated {
			company_id: u32,
			old_key_id: Vec<u8>,
			new_key_id: Vec<u8>,
		},
		/// Data visibility updated
		VisibilityUpdated {
			data_id: [u8; 32],
			new_visibility: VisibilityLevel,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Data not found
		DataNotFound,
		/// Not authorized to access data
		NotAuthorized,
		/// Not the data owner
		NotOwner,
		/// Encrypted data too long
		EncryptedDataTooLong,
		/// Key ID too long
		KeyIdTooLong,
		/// Invalid encryption metadata
		InvalidMetadata,
		/// Company key not found
		CompanyKeyNotFound,
		/// Key already exists
		KeyAlreadyExists,
		/// No active key for company
		NoActiveKey,
		/// Too many authorized roles
		TooManyAuthorizedRoles,
		/// Data already exists
		DataAlreadyExists,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a company encryption key
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn register_company_key(
			origin: OriginFor<T>,
			company_id: u32,
			key_id: Vec<u8>,
			key_hash: [u8; 32],
			algorithm: EncryptionAlgorithm,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_key_id: BoundedVec<u8, T::MaxKeyIdLength> =
				key_id.clone().try_into().map_err(|_| Error::<T>::KeyIdTooLong)?;

			// Check if key already exists
			ensure!(
				!CompanyKeys::<T>::contains_key(company_id, &bounded_key_id),
				Error::<T>::KeyAlreadyExists
			);

			let now = frame_system::Pallet::<T>::block_number();

			let key_info = CompanyKeyInfo {
				key_id: bounded_key_id.clone(),
				key_hash,
				algorithm,
				is_active: true,
				created_at: now,
			};

			CompanyKeys::<T>::insert(company_id, &bounded_key_id, key_info);

			// Set as active key if no active key exists
			if !ActiveCompanyKey::<T>::contains_key(company_id) {
				ActiveCompanyKey::<T>::insert(company_id, bounded_key_id);
			}

			Self::deposit_event(Event::CompanyKeyRegistered { company_id, key_id });

			Ok(())
		}

		/// Store encrypted data
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn store_encrypted_data(
			origin: OriginFor<T>,
			ciphertext: Vec<u8>,
			plaintext_hash: [u8; 32],
			company_id: u32,
			key_id: Vec<u8>,
			algorithm: EncryptionAlgorithm,
			iv: Option<[u8; 16]>,
			visibility: VisibilityLevel,
			authorized_roles: Vec<u32>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_ciphertext: BoundedVec<u8, T::MaxEncryptedDataLength> =
				ciphertext.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			let bounded_key_id: BoundedVec<u8, T::MaxKeyIdLength> =
				key_id.try_into().map_err(|_| Error::<T>::KeyIdTooLong)?;

			let bounded_roles: BoundedVec<u32, T::MaxAuthorizedRoles> =
				authorized_roles.try_into().map_err(|_| Error::<T>::TooManyAuthorizedRoles)?;

			// Verify company key exists
			ensure!(
				CompanyKeys::<T>::contains_key(company_id, &bounded_key_id),
				Error::<T>::CompanyKeyNotFound
			);

			// Generate unique data ID
			let data_id = Self::generate_data_id(&who, &plaintext_hash, company_id);

			// Check if data already exists
			ensure!(
				!EncryptedData::<T>::contains_key(data_id),
				Error::<T>::DataAlreadyExists
			);

			let now = frame_system::Pallet::<T>::block_number();

			let metadata = EncryptionMetadata {
				algorithm,
				key_id: bounded_key_id,
				salt: None,
				iv,
			};

			let entry = EncryptedDataEntry {
				ciphertext: bounded_ciphertext,
				plaintext_hash,
				metadata,
				owner: who.clone(),
				company_id,
				visibility: visibility.clone(),
				authorized_roles: bounded_roles,
				created_at: now,
				updated_at: now,
			};

			EncryptedData::<T>::insert(data_id, entry);

			Self::deposit_event(Event::EncryptedDataStored {
				data_id,
				owner: who,
				company_id,
				visibility,
			});

			Ok(())
		}

		/// Update encrypted data
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn update_encrypted_data(
			origin: OriginFor<T>,
			data_id: [u8; 32],
			new_ciphertext: Vec<u8>,
			new_plaintext_hash: [u8; 32],
			new_iv: Option<[u8; 16]>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_ciphertext: BoundedVec<u8, T::MaxEncryptedDataLength> =
				new_ciphertext.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			EncryptedData::<T>::try_mutate(data_id, |maybe_entry| -> DispatchResult {
				let entry = maybe_entry.as_mut().ok_or(Error::<T>::DataNotFound)?;

				// Only owner can update
				ensure!(entry.owner == who, Error::<T>::NotOwner);

				entry.ciphertext = bounded_ciphertext;
				entry.plaintext_hash = new_plaintext_hash;
				entry.metadata.iv = new_iv;
				entry.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::EncryptedDataUpdated {
				data_id,
				updater: who,
			});

			Ok(())
		}

		/// Get encrypted data (with access control check)
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn access_encrypted_data(
			origin: OriginFor<T>,
			data_id: [u8; 32],
			user_role: u32,
			user_company_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let entry = EncryptedData::<T>::get(data_id).ok_or(Error::<T>::DataNotFound)?;

			// Access control check
			let has_access = Self::check_access(&who, &entry, user_role, user_company_id);
			ensure!(has_access, Error::<T>::NotAuthorized);

			// Log access
			let now = frame_system::Pallet::<T>::block_number();
			AccessLog::<T>::insert((data_id, who.clone(), now), ());

			Self::deposit_event(Event::EncryptedDataAccessed {
				data_id,
				accessor: who,
			});

			Ok(())
		}

		/// Update visibility level
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn update_visibility(
			origin: OriginFor<T>,
			data_id: [u8; 32],
			new_visibility: VisibilityLevel,
			new_authorized_roles: Vec<u32>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_roles: BoundedVec<u32, T::MaxAuthorizedRoles> =
				new_authorized_roles.try_into().map_err(|_| Error::<T>::TooManyAuthorizedRoles)?;

			EncryptedData::<T>::try_mutate(data_id, |maybe_entry| -> DispatchResult {
				let entry = maybe_entry.as_mut().ok_or(Error::<T>::DataNotFound)?;

				// Only owner can update visibility
				ensure!(entry.owner == who, Error::<T>::NotOwner);

				entry.visibility = new_visibility.clone();
				entry.authorized_roles = bounded_roles;
				entry.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::VisibilityUpdated {
				data_id,
				new_visibility,
			});

			Ok(())
		}

		/// Rotate company encryption key
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn rotate_company_key(
			origin: OriginFor<T>,
			company_id: u32,
			new_key_id: Vec<u8>,
			new_key_hash: [u8; 32],
			algorithm: EncryptionAlgorithm,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let bounded_new_key_id: BoundedVec<u8, T::MaxKeyIdLength> =
				new_key_id.clone().try_into().map_err(|_| Error::<T>::KeyIdTooLong)?;

			// Get old active key
			let old_key_id = ActiveCompanyKey::<T>::get(company_id)
				.ok_or(Error::<T>::NoActiveKey)?;

			// Deactivate old key
			CompanyKeys::<T>::mutate(company_id, &old_key_id, |maybe_key| {
				if let Some(key) = maybe_key {
					key.is_active = false;
				}
			});

			// Register new key
			let now = frame_system::Pallet::<T>::block_number();
			let new_key_info = CompanyKeyInfo {
				key_id: bounded_new_key_id.clone(),
				key_hash: new_key_hash,
				algorithm,
				is_active: true,
				created_at: now,
			};

			CompanyKeys::<T>::insert(company_id, &bounded_new_key_id, new_key_info);
			ActiveCompanyKey::<T>::insert(company_id, bounded_new_key_id);

			Self::deposit_event(Event::CompanyKeyRotated {
				company_id,
				old_key_id: old_key_id.to_vec(),
				new_key_id,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Generate unique data ID
		fn generate_data_id(
			owner: &T::AccountId,
			plaintext_hash: &[u8; 32],
			company_id: u32,
		) -> [u8; 32] {
			use frame::traits::Hash;
			let data = (owner, plaintext_hash, company_id);
			let hash = T::Hashing::hash_of(&data);
			let mut result = [0u8; 32];
			result.copy_from_slice(hash.as_ref());
			result
		}

		/// Check if user has access to data
		fn check_access(
			who: &T::AccountId,
			entry: &EncryptedDataEntry<T>,
			user_role: u32,
			user_company_id: u32,
		) -> bool {
			// Owner always has access
			if entry.owner == *who {
				return true;
			}

			match entry.visibility {
				VisibilityLevel::Public => true,
				VisibilityLevel::Company => user_company_id == entry.company_id,
				VisibilityLevel::Management => {
					// Role IDs: 1=Admin, 2=Manager (example)
					user_company_id == entry.company_id && (user_role == 1 || user_role == 2)
				}
				VisibilityLevel::Restricted => {
					user_company_id == entry.company_id
						&& entry.authorized_roles.contains(&user_role)
				}
				VisibilityLevel::Private => false,
			}
		}
	}
}
