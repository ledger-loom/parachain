//! # Business Management Pallet
//!
//! A FRAME pallet for managing businesses in a decentralized system.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating and managing businesses
//! - Inviting team members via email/wallet
//! - Business settings and configuration (encrypted on-chain)
//! - Ownership transfer between users
//! - Business verification and status management
//!
//! ## Encryption Model
//!
//! Businesses support dual encryption types:
//! - **ProjectKey**: Data encrypted by core platform with master key (for email users)
//! - **WalletKey**: Data encrypted client-side with user's wallet key (for wallet users)
//!
//! All business data is stored in the core database and selectively synced to blockchain for immutability.

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

#[frame::pallet(dev_mode)]
pub mod pallet {
	use crate::WeightInfo;
	use frame::prelude::*;
	use frame::deps::codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::prelude::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxBusinessNameLength: Get<u32>;

		#[pallet::constant]
		type MaxMembers: Get<u32>;

		#[pallet::constant]
		type MaxPendingInvites: Get<u32>;

		#[pallet::constant]
		type MaxConfigDataLength: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Encryption type for business data
	#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum EncryptionType {
		/// Encrypted by core platform with project master key (for email users)
		ProjectKey,
		/// Encrypted client-side with user's wallet key (for wallet users)
		WalletKey,
	}

	/// Business information (minimal on-chain footprint)
	/// Full business data stored in core database
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Business<T: Config> {
		/// Business name
		pub name: BoundedVec<u8, T::MaxBusinessNameLength>,
		/// Business owner (user account)
		pub owner: T::AccountId,
		/// Business UUID (reference to core database - 16 bytes)
		pub business_uuid: [u8; 16],
		/// Encryption type for this business's data
		pub encryption_type: EncryptionType,
		/// Creation timestamp
		pub created_at: BlockNumberFor<T>,
		/// Verification status
		pub is_verified: bool,
		/// Number of members
		pub member_count: u32,
		/// Encrypted configuration data (statuses, categories, params, roles)
		pub encrypted_config: Option<BoundedVec<u8, T::MaxConfigDataLength>>,
	}

	/// Business member details
	#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum MemberRole {
		Owner,
		Manager,
		Warehouse,
		Transport,
		Supplier,
	}

	/// Invitation status
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum InviteStatus {
		Pending,
		Accepted,
		Rejected,
		Expired,
	}

	/// Business invitation
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Invitation<T: Config> {
		pub business_uuid: [u8; 16],
		pub invitee: T::AccountId,
		pub role: MemberRole,
		pub status: InviteStatus,
		pub invited_at: BlockNumberFor<T>,
	}

	/// Storage: Businesses indexed by UUID
	#[pallet::storage]
	#[pallet::getter(fn businesses)]
	pub type Businesses<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 16], Business<T>>;

	/// Storage: Business members (indexed by UUID)
	#[pallet::storage]
	#[pallet::getter(fn business_members)]
	pub type BusinessMembers<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, [u8; 16], Blake2_128Concat, T::AccountId, MemberRole>;

	/// Storage: User's businesses (indexed by UUID)
	#[pallet::storage]
	#[pallet::getter(fn user_businesses)]
	pub type UserBusinesses<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, [u8; 16], ()>;

	/// Storage: Invitations (indexed by UUID)
	#[pallet::storage]
	#[pallet::getter(fn invitations)]
	pub type Invitations<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, [u8; 16], Blake2_128Concat, T::AccountId, Invitation<T>>;

	/// Events for the pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Business synced from core database to blockchain
		BusinessCreated {
			business_uuid: [u8; 16],
			owner: T::AccountId,
			encryption_type: EncryptionType,
			name: Vec<u8>,
		},
		/// Member invited to business
		MemberInvited { business_uuid: [u8; 16], invitee: T::AccountId, role: MemberRole },
		/// Invitation accepted
		InvitationAccepted { business_uuid: [u8; 16], member: T::AccountId },
		/// Invitation rejected
		InvitationRejected { business_uuid: [u8; 16], invitee: T::AccountId },
		/// Member removed from business
		MemberRemoved { business_uuid: [u8; 16], member: T::AccountId },
		/// Ownership transferred
		OwnershipTransferred { business_uuid: [u8; 16], old_owner: T::AccountId, new_owner: T::AccountId },
		/// Business settings updated
		BusinessSettingsUpdated { business_uuid: [u8; 16] },
		/// Business verified
		BusinessVerified { business_uuid: [u8; 16] },
		/// Business configuration updated (encrypted)
		BusinessConfigUpdated { business_uuid: [u8; 16] },
	}

	/// Errors for the pallet
	#[pallet::error]
	pub enum Error<T> {
		/// Business not found
		BusinessNotFound,
		/// Not the business owner
		NotBusinessOwner,
		/// Not a member of the business
		NotMember,
		/// Member already exists
		MemberAlreadyExists,
		/// Invitation not found
		InvitationNotFound,
		/// Invitation already exists
		InvitationAlreadyExists,
		/// Too many members
		TooManyMembers,
		/// Too many pending invites
		TooManyPendingInvites,
		/// Business name too long
		BusinessNameTooLong,
		/// Config data too long
		ConfigDataTooLong,
		/// Invalid invitation status
		InvalidInvitationStatus,
		/// Cannot remove owner
		CannotRemoveOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Dispatchable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sync a business from core database to blockchain
		///
		/// This is called by the core platform via secure channel to store business data on-chain.
		/// Business is created in core database first, then synced to blockchain for immutability.
		///
		/// Parameters:
		/// - name: Business name
		/// - business_uuid: UUID from core database (16 bytes)
		/// - encryption_type: ProjectKey or WalletKey
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_business())]
		pub fn create_business(
			origin: OriginFor<T>,
			name: Vec<u8>,
			business_uuid: [u8; 16],
			encryption_type: EncryptionType,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate business name
			let bounded_name: BoundedVec<u8, T::MaxBusinessNameLength> =
				name.clone().try_into().map_err(|_| Error::<T>::BusinessNameTooLong)?;

			let now = frame_system::Pallet::<T>::block_number();

			// Create business record (minimal on-chain footprint)
			let business = Business {
				name: bounded_name,
				owner: who.clone(),
				business_uuid,
				encryption_type: encryption_type.clone(),
				created_at: now,
				is_verified: false,
				member_count: 1,
				encrypted_config: None,
			};

			// Store business
			Businesses::<T>::insert(business_uuid, business);

			// Add owner as member
			BusinessMembers::<T>::insert(business_uuid, &who, MemberRole::Owner);

			// Add to user's businesses
			UserBusinesses::<T>::insert(&who, business_uuid, ());

			// Emit event
			Self::deposit_event(Event::BusinessCreated {
				business_uuid,
				owner: who,
				encryption_type,
				name,
			});

			Ok(())
		}

		/// Update business encrypted configuration
		///
		/// The config is encrypted based on encryption_type:
		/// - ProjectKey: Encrypted by core with master key
		/// - WalletKey: Encrypted client-side with user's wallet
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_business())]
		pub fn update_business_config(
			origin: OriginFor<T>,
			business_uuid: [u8; 16],
			encrypted_config: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate config size
			let bounded_config: BoundedVec<u8, T::MaxConfigDataLength> =
				encrypted_config.try_into().map_err(|_| Error::<T>::ConfigDataTooLong)?;

			// Update business config
			Businesses::<T>::try_mutate(business_uuid, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;

				// Ensure caller is the owner
				ensure!(business.owner == who, Error::<T>::NotBusinessOwner);

				// Update encrypted config
				business.encrypted_config = Some(bounded_config);

				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::BusinessConfigUpdated { business_uuid });

			Ok(())
		}

		/// Invite a member to the business
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::invite_member())]
		pub fn invite_member(
			origin: OriginFor<T>,
			business_uuid: [u8; 16],
			invitee: T::AccountId,
			role: MemberRole,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get business
			let _business = Businesses::<T>::get(business_uuid).ok_or(Error::<T>::BusinessNotFound)?;

			// Ensure caller is owner or manager
			let caller_role = BusinessMembers::<T>::get(business_uuid, &who)
				.ok_or(Error::<T>::NotMember)?;
			ensure!(
				matches!(caller_role, MemberRole::Owner | MemberRole::Manager),
				Error::<T>::NotBusinessOwner
			);

			// Ensure invitee is not already a member
			ensure!(
				!BusinessMembers::<T>::contains_key(business_uuid, &invitee),
				Error::<T>::MemberAlreadyExists
			);

			// Ensure invitation doesn't already exist
			ensure!(
				!Invitations::<T>::contains_key(business_uuid, &invitee),
				Error::<T>::InvitationAlreadyExists
			);

			let now = frame_system::Pallet::<T>::block_number();

			// Create invitation
			let invitation = Invitation {
				business_uuid,
				invitee: invitee.clone(),
				role: role.clone(),
				status: InviteStatus::Pending,
				invited_at: now,
			};

			// Store invitation
			Invitations::<T>::insert(business_uuid, &invitee, invitation);

			// Emit event
			Self::deposit_event(Event::MemberInvited {
				business_uuid,
				invitee,
				role,
			});

			Ok(())
		}

		/// Accept an invitation to join a business
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::accept_invitation())]
		pub fn accept_invitation(origin: OriginFor<T>, business_uuid: [u8; 16]) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get invitation
			let invitation = Invitations::<T>::get(business_uuid, &who)
				.ok_or(Error::<T>::InvitationNotFound)?;

			// Ensure invitation is pending
			ensure!(
				invitation.status == InviteStatus::Pending,
				Error::<T>::InvalidInvitationStatus
			);

			// Add member to business
			BusinessMembers::<T>::insert(business_uuid, &who, invitation.role);

			// Add to user's businesses
			UserBusinesses::<T>::insert(&who, business_uuid, ());

			// Increment member count
			Businesses::<T>::try_mutate(business_uuid, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;
				business.member_count = business.member_count.saturating_add(1);
				Ok(())
			})?;

			// Update invitation status
			Invitations::<T>::try_mutate(business_uuid, &who, |maybe_invite| -> DispatchResult {
				let invite = maybe_invite.as_mut().ok_or(Error::<T>::InvitationNotFound)?;
				invite.status = InviteStatus::Accepted;
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::InvitationAccepted {
				business_uuid,
				member: who,
			});

			Ok(())
		}

		/// Reject an invitation
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::reject_invitation())]
		pub fn reject_invitation(origin: OriginFor<T>, business_uuid: [u8; 16]) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Update invitation status
			Invitations::<T>::try_mutate(business_uuid, &who, |maybe_invite| -> DispatchResult {
				let invite = maybe_invite.as_mut().ok_or(Error::<T>::InvitationNotFound)?;
				ensure!(
					invite.status == InviteStatus::Pending,
					Error::<T>::InvalidInvitationStatus
				);
				invite.status = InviteStatus::Rejected;
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::InvitationRejected {
				business_uuid,
				invitee: who,
			});

			Ok(())
		}

		/// Remove a member from the business
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_member())]
		pub fn remove_member(
			origin: OriginFor<T>,
			business_uuid: [u8; 16],
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get business
			let business = Businesses::<T>::get(business_uuid).ok_or(Error::<T>::BusinessNotFound)?;

			// Ensure caller is owner
			ensure!(business.owner == who, Error::<T>::NotBusinessOwner);

			// Ensure not removing owner
			ensure!(member != business.owner, Error::<T>::CannotRemoveOwner);

			// Ensure member exists
			ensure!(
				BusinessMembers::<T>::contains_key(business_uuid, &member),
				Error::<T>::NotMember
			);

			// Remove member
			BusinessMembers::<T>::remove(business_uuid, &member);
			UserBusinesses::<T>::remove(&member, business_uuid);

			// Decrement member count
			Businesses::<T>::try_mutate(business_uuid, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;
				business.member_count = business.member_count.saturating_sub(1);
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::MemberRemoved { business_uuid, member });

			Ok(())
		}

		/// Transfer business ownership
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transfer_ownership())]
		pub fn transfer_ownership(
			origin: OriginFor<T>,
			business_uuid: [u8; 16],
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure new owner is a member
			ensure!(
				BusinessMembers::<T>::contains_key(business_uuid, &new_owner),
				Error::<T>::NotMember
			);

			// Update business owner
			Businesses::<T>::try_mutate(business_uuid, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;

				// Ensure caller is current owner
				ensure!(business.owner == who, Error::<T>::NotBusinessOwner);

				// Update owner
				let old_owner = business.owner.clone();
				business.owner = new_owner.clone();

				// Update roles
				BusinessMembers::<T>::insert(business_uuid, &new_owner, MemberRole::Owner);
				BusinessMembers::<T>::insert(business_uuid, &old_owner, MemberRole::Manager);

				// Emit event
				Self::deposit_event(Event::OwnershipTransferred {
					business_uuid,
					old_owner,
					new_owner,
				});

				Ok(())
			})
		}

		/// Verify a business (requires root/sudo)
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::verify_business())]
		pub fn verify_business(origin: OriginFor<T>, business_uuid: [u8; 16]) -> DispatchResult {
			ensure_root(origin)?;

			// Update verification status
			Businesses::<T>::try_mutate(business_uuid, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;
				business.is_verified = true;
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::BusinessVerified { business_uuid });

			Ok(())
		}
	}
}
