//! # User Management Pallet
//!
//! A FRAME pallet for managing users in a supply chain system.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - User registration with email/password + wallet
//! - User profiles and settings
//! - Email-wallet linking
//! - KYC/identity verification process
//!
//! ## Dispatchable Functions
//!
//! - `register_user` - Register a new user with email and wallet
//! - `link_email` - Link email to wallet address
//! - `link_wallet` - Link wallet to existing user
//! - `update_profile` - Update user profile information
//! - `submit_verification` - Submit KYC verification documents
//! - `approve_verification` - Approve verification request (privileged)
//! - `reject_verification` - Reject verification request (privileged)

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame::pallet]
pub mod pallet {
	use frame::prelude::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Maximum length of user profile fields
		#[pallet::constant]
		type MaxProfileLength: Get<u32>;

		/// Maximum number of verification documents per request
		#[pallet::constant]
		type MaxDocuments: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// User profile information
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct UserProfile<T: Config> {
		/// User's display name
		pub name: BoundedVec<u8, T::MaxProfileLength>,
		/// User's email (hashed for privacy)
		pub email_hash: [u8; 32],
		/// Account ID (wallet address)
		pub account_id: T::AccountId,
		/// Verification status
		pub is_verified: bool,
		/// Registration timestamp
		pub registered_at: BlockNumberFor<T>,
		/// Last update timestamp
		pub updated_at: BlockNumberFor<T>,
	}

	/// Verification request details
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct VerificationRequest<T: Config> {
		/// User account ID
		pub user: T::AccountId,
		/// Verification type
		pub verification_type: VerificationType,
		/// Document hashes
		pub document_hashes: BoundedVec<[u8; 32], T::MaxDocuments>,
		/// Request status
		pub status: VerificationStatus,
		/// Submission timestamp
		pub submitted_at: BlockNumberFor<T>,
		/// Review timestamp
		pub reviewed_at: Option<BlockNumberFor<T>>,
	}

	/// Types of verification
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VerificationType {
		/// Identity verification (KYC)
		Identity,
		/// Business verification
		Business,
		/// Address verification
		Address,
	}

	/// Verification status
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VerificationStatus {
		/// Pending review
		Pending,
		/// Approved
		Approved,
		/// Rejected
		Rejected,
		/// Requires more information
		RequiresMoreInfo,
	}

	/// Storage: Users indexed by account ID
	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub type Users<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, UserProfile<T>>;

	/// Storage: Email hash to account ID mapping
	#[pallet::storage]
	#[pallet::getter(fn email_to_account)]
	pub type EmailToAccount<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId>;

	/// Storage: Verification requests
	#[pallet::storage]
	#[pallet::getter(fn verification_requests)]
	pub type VerificationRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		VerificationRequest<T>,
	>;

	/// Storage: Total user count
	#[pallet::storage]
	#[pallet::getter(fn user_count)]
	pub type UserCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Events for the pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// User registered successfully
		UserRegistered { account: T::AccountId, email_hash: [u8; 32] },
		/// Email linked to account
		EmailLinked { account: T::AccountId, email_hash: [u8; 32] },
		/// Wallet linked to account
		WalletLinked { account: T::AccountId },
		/// Profile updated
		ProfileUpdated { account: T::AccountId },
		/// Verification submitted
		VerificationSubmitted { account: T::AccountId, verification_type: VerificationType },
		/// Verification approved
		VerificationApproved { account: T::AccountId },
		/// Verification rejected
		VerificationRejected { account: T::AccountId },
	}

	/// Errors for the pallet
	#[pallet::error]
	pub enum Error<T> {
		/// User already exists
		UserAlreadyExists,
		/// User not found
		UserNotFound,
		/// Email already registered
		EmailAlreadyRegistered,
		/// Wallet already registered
		WalletAlreadyRegistered,
		/// Verification request not found
		VerificationNotFound,
		/// Verification already approved
		AlreadyVerified,
		/// Invalid verification status
		InvalidVerificationStatus,
		/// Profile data too long
		ProfileTooLong,
		/// Too many documents
		TooManyDocuments,
		/// Not authorized
		NotAuthorized,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Dispatchable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new user
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register_user())]
		pub fn register_user(
			origin: OriginFor<T>,
			name: Vec<u8>,
			email_hash: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure user doesn't already exist
			ensure!(!Users::<T>::contains_key(&who), Error::<T>::UserAlreadyExists);

			// Ensure email not already registered
			ensure!(
				!EmailToAccount::<T>::contains_key(&email_hash),
				Error::<T>::EmailAlreadyRegistered
			);

			// Convert name to bounded vec
			let bounded_name: BoundedVec<u8, T::MaxProfileLength> =
				name.try_into().map_err(|_| Error::<T>::ProfileTooLong)?;

			let now = frame_system::Pallet::<T>::block_number();

			// Create user profile
			let profile = UserProfile {
				name: bounded_name,
				email_hash,
				account_id: who.clone(),
				is_verified: false,
				registered_at: now,
				updated_at: now,
			};

			// Store user profile
			Users::<T>::insert(&who, profile);

			// Store email to account mapping
			EmailToAccount::<T>::insert(email_hash, &who);

			// Increment user count
			UserCount::<T>::mutate(|count| *count = count.saturating_add(1));

			// Emit event
			Self::deposit_event(Event::UserRegistered {
				account: who,
				email_hash,
			});

			Ok(())
		}

		/// Update user profile
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_profile())]
		pub fn update_profile(
			origin: OriginFor<T>,
			name: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get existing profile
			Users::<T>::try_mutate(&who, |maybe_profile| -> DispatchResult {
				let profile = maybe_profile.as_mut().ok_or(Error::<T>::UserNotFound)?;

				// Update name if provided
				if let Some(new_name) = name {
					let bounded_name: BoundedVec<u8, T::MaxProfileLength> =
						new_name.try_into().map_err(|_| Error::<T>::ProfileTooLong)?;
					profile.name = bounded_name;
				}

				// Update timestamp
				profile.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::ProfileUpdated { account: who });

			Ok(())
		}

		/// Submit verification request
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::submit_verification())]
		pub fn submit_verification(
			origin: OriginFor<T>,
			verification_type: VerificationType,
			document_hashes: Vec<[u8; 32]>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure user exists
			ensure!(Users::<T>::contains_key(&who), Error::<T>::UserNotFound);

			// Convert to bounded vec
			let bounded_docs: BoundedVec<[u8; 32], T::MaxDocuments> =
				document_hashes.try_into().map_err(|_| Error::<T>::TooManyDocuments)?;

			let now = frame_system::Pallet::<T>::block_number();

			// Create verification request
			let request = VerificationRequest {
				user: who.clone(),
				verification_type: verification_type.clone(),
				document_hashes: bounded_docs,
				status: VerificationStatus::Pending,
				submitted_at: now,
				reviewed_at: None,
			};

			// Store verification request
			VerificationRequests::<T>::insert(&who, request);

			// Emit event
			Self::deposit_event(Event::VerificationSubmitted {
				account: who,
				verification_type,
			});

			Ok(())
		}

		/// Approve verification request (requires root/sudo)
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::approve_verification())]
		pub fn approve_verification(
			origin: OriginFor<T>,
			user: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Update verification request
			VerificationRequests::<T>::try_mutate(&user, |maybe_request| -> DispatchResult {
				let request = maybe_request.as_mut().ok_or(Error::<T>::VerificationNotFound)?;

				request.status = VerificationStatus::Approved;
				request.reviewed_at = Some(frame_system::Pallet::<T>::block_number());

				Ok(())
			})?;

			// Update user profile
			Users::<T>::try_mutate(&user, |maybe_profile| -> DispatchResult {
				let profile = maybe_profile.as_mut().ok_or(Error::<T>::UserNotFound)?;
				profile.is_verified = true;
				profile.updated_at = frame_system::Pallet::<T>::block_number();
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::VerificationApproved { account: user });

			Ok(())
		}

		/// Reject verification request (requires root/sudo)
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::reject_verification())]
		pub fn reject_verification(
			origin: OriginFor<T>,
			user: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Update verification request
			VerificationRequests::<T>::try_mutate(&user, |maybe_request| -> DispatchResult {
				let request = maybe_request.as_mut().ok_or(Error::<T>::VerificationNotFound)?;

				request.status = VerificationStatus::Rejected;
				request.reviewed_at = Some(frame_system::Pallet::<T>::block_number());

				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::VerificationRejected { account: user });

			Ok(())
		}
	}
}
