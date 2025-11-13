//! # Business Management Pallet
//!
//! A FRAME pallet for managing businesses in a decentralized system.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating and managing businesses with hierarchical wallet derivation
//! - Inviting team members via email/wallet
//! - Business settings and configuration (encrypted on-business)
//! - Ownership transfer between users
//! - Business verification and status management
//!
//! ## Hierarchical Wallet Derivation
//!
//! Each business has a derived wallet from the user's wallet:
//! - User wallet: m/44'/354'/user_index'
//! - Business wallet: user_wallet/0/business_id'
//!
//! This enables:
//! - Business-specific encryption keys
//! - Separate key management per business
//! - Secure multi-business support

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

	/// Business/Supply Business information
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Business<T: Config> {
		/// Business name
		pub name: BoundedVec<u8, T::MaxBusinessNameLength>,
		/// Business owner (user account)
		pub owner: T::AccountId,
		/// Owner's user index (for wallet derivation)
		pub owner_user_index: u32,
		/// Business ID (used for derivation path)
		pub business_id: u32,
		/// Derived public key for this business (32 bytes)
		pub derived_public_key: [u8; 32],
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
		pub business_id: u32,
		pub invitee: T::AccountId,
		pub role: MemberRole,
		pub status: InviteStatus,
		pub invited_at: BlockNumberFor<T>,
	}

	/// Storage: Businesss indexed by ID
	#[pallet::storage]
	#[pallet::getter(fn businesss)]
	pub type Businesss<T: Config> = StorageMap<_, Blake2_128Concat, u32, Business<T>>;

	/// Storage: Business members
	#[pallet::storage]
	#[pallet::getter(fn business_members)]
	pub type BusinessMembers<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, MemberRole>;

	/// Storage: User's businesss
	#[pallet::storage]
	#[pallet::getter(fn user_businesss)]
	pub type UserBusinesss<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, u32, ()>;

	/// Storage: Invitations
	#[pallet::storage]
	#[pallet::getter(fn invitations)]
	pub type Invitations<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, Invitation<T>>;

	/// Storage: Total business count
	#[pallet::storage]
	#[pallet::getter(fn business_count)]
	pub type BusinessCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Events for the pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Business created with derived wallet
		BusinessCreated {
			business_id: u32,
			owner: T::AccountId,
			owner_user_index: u32,
			derived_public_key: [u8; 32],
			name: Vec<u8>,
		},
		/// Member invited to business
		MemberInvited { business_id: u32, invitee: T::AccountId, role: MemberRole },
		/// Invitation accepted
		InvitationAccepted { business_id: u32, member: T::AccountId },
		/// Invitation rejected
		InvitationRejected { business_id: u32, invitee: T::AccountId },
		/// Member removed from business
		MemberRemoved { business_id: u32, member: T::AccountId },
		/// Ownership transferred
		OwnershipTransferred { business_id: u32, old_owner: T::AccountId, new_owner: T::AccountId },
		/// Business settings updated
		BusinessSettingsUpdated { business_id: u32 },
		/// Business verified
		BusinessVerified { business_id: u32 },
		/// Business configuration updated (encrypted)
		BusinessConfigUpdated { business_id: u32 },
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
		/// Create a new business with hierarchical wallet derivation
		///
		/// The derived_public_key should be generated off-business using:
		/// user_wallet/0/business_id' derivation path
		///
		/// Parameters:
		/// - name: Business name
		/// - owner_user_index: The user's index (from user-management pallet)
		/// - derived_public_key: The derived public key for this business
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_business())]
		pub fn create_business(
			origin: OriginFor<T>,
			name: Vec<u8>,
			owner_user_index: u32,
			derived_public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate business name
			let bounded_name: BoundedVec<u8, T::MaxBusinessNameLength> =
				name.clone().try_into().map_err(|_| Error::<T>::BusinessNameTooLong)?;

			// Get next business ID
			let business_id = BusinessCount::<T>::get();

			let now = frame_system::Pallet::<T>::block_number();

			// Create business with derived wallet info
			let business = Business {
				name: bounded_name,
				owner: who.clone(),
				owner_user_index,
				business_id,
				derived_public_key,
				created_at: now,
				is_verified: false,
				member_count: 1,
				encrypted_config: None,
			};

			// Store business
			Businesss::<T>::insert(business_id, business);

			// Add owner as member
			BusinessMembers::<T>::insert(business_id, &who, MemberRole::Owner);

			// Add to user's businesss
			UserBusinesss::<T>::insert(&who, business_id, ());

			// Increment business count
			BusinessCount::<T>::mutate(|count| *count = count.saturating_add(1));

			// Emit event
			Self::deposit_event(Event::BusinessCreated {
				business_id,
				owner: who,
				owner_user_index,
				derived_public_key,
				name,
			});

			Ok(())
		}

		/// Update business encrypted configuration
		///
		/// The config should be encrypted off-business with the business's public key
		/// and include: statuses, categories, custom params, roles, permissions
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_business_config())]
		pub fn update_business_config(
			origin: OriginFor<T>,
			business_id: u32,
			encrypted_config: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate config size
			let bounded_config: BoundedVec<u8, T::MaxConfigDataLength> =
				encrypted_config.try_into().map_err(|_| Error::<T>::ConfigDataTooLong)?;

			// Update business config
			Businesss::<T>::try_mutate(business_id, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;

				// Ensure caller is the owner
				ensure!(business.owner == who, Error::<T>::NotBusinessOwner);

				// Update encrypted config
				business.encrypted_config = Some(bounded_config);

				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::BusinessConfigUpdated { business_id });

			Ok(())
		}

		/// Invite a member to the business
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::invite_member())]
		pub fn invite_member(
			origin: OriginFor<T>,
			business_id: u32,
			invitee: T::AccountId,
			role: MemberRole,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get business
			let business = Businesss::<T>::get(business_id).ok_or(Error::<T>::BusinessNotFound)?;

			// Ensure caller is owner or manager
			let caller_role = BusinessMembers::<T>::get(business_id, &who)
				.ok_or(Error::<T>::NotMember)?;
			ensure!(
				matches!(caller_role, MemberRole::Owner | MemberRole::Manager),
				Error::<T>::NotBusinessOwner
			);

			// Ensure invitee is not already a member
			ensure!(
				!BusinessMembers::<T>::contains_key(business_id, &invitee),
				Error::<T>::MemberAlreadyExists
			);

			// Ensure invitation doesn't already exist
			ensure!(
				!Invitations::<T>::contains_key(business_id, &invitee),
				Error::<T>::InvitationAlreadyExists
			);

			let now = frame_system::Pallet::<T>::block_number();

			// Create invitation
			let invitation = Invitation {
				business_id,
				invitee: invitee.clone(),
				role: role.clone(),
				status: InviteStatus::Pending,
				invited_at: now,
			};

			// Store invitation
			Invitations::<T>::insert(business_id, &invitee, invitation);

			// Emit event
			Self::deposit_event(Event::MemberInvited {
				business_id,
				invitee,
				role,
			});

			Ok(())
		}

		/// Accept an invitation to join a business
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::accept_invitation())]
		pub fn accept_invitation(origin: OriginFor<T>, business_id: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get invitation
			let invitation = Invitations::<T>::get(business_id, &who)
				.ok_or(Error::<T>::InvitationNotFound)?;

			// Ensure invitation is pending
			ensure!(
				invitation.status == InviteStatus::Pending,
				Error::<T>::InvalidInvitationStatus
			);

			// Add member to business
			BusinessMembers::<T>::insert(business_id, &who, invitation.role);

			// Add to user's businesss
			UserBusinesss::<T>::insert(&who, business_id, ());

			// Increment member count
			Businesss::<T>::try_mutate(business_id, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;
				business.member_count = business.member_count.saturating_add(1);
				Ok(())
			})?;

			// Update invitation status
			Invitations::<T>::try_mutate(business_id, &who, |maybe_invite| -> DispatchResult {
				let invite = maybe_invite.as_mut().ok_or(Error::<T>::InvitationNotFound)?;
				invite.status = InviteStatus::Accepted;
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::InvitationAccepted {
				business_id,
				member: who,
			});

			Ok(())
		}

		/// Reject an invitation
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::reject_invitation())]
		pub fn reject_invitation(origin: OriginFor<T>, business_id: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Update invitation status
			Invitations::<T>::try_mutate(business_id, &who, |maybe_invite| -> DispatchResult {
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
				business_id,
				invitee: who,
			});

			Ok(())
		}

		/// Remove a member from the business
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_member())]
		pub fn remove_member(
			origin: OriginFor<T>,
			business_id: u32,
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get business
			let business = Businesss::<T>::get(business_id).ok_or(Error::<T>::BusinessNotFound)?;

			// Ensure caller is owner
			ensure!(business.owner == who, Error::<T>::NotBusinessOwner);

			// Ensure not removing owner
			ensure!(member != business.owner, Error::<T>::CannotRemoveOwner);

			// Ensure member exists
			ensure!(
				BusinessMembers::<T>::contains_key(business_id, &member),
				Error::<T>::NotMember
			);

			// Remove member
			BusinessMembers::<T>::remove(business_id, &member);
			UserBusinesss::<T>::remove(&member, business_id);

			// Decrement member count
			Businesss::<T>::try_mutate(business_id, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;
				business.member_count = business.member_count.saturating_sub(1);
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::MemberRemoved { business_id, member });

			Ok(())
		}

		/// Transfer business ownership
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transfer_ownership())]
		pub fn transfer_ownership(
			origin: OriginFor<T>,
			business_id: u32,
			new_owner: T::AccountId,
			new_owner_user_index: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure new owner is a member
			ensure!(
				BusinessMembers::<T>::contains_key(business_id, &new_owner),
				Error::<T>::NotMember
			);

			// Update business owner
			Businesss::<T>::try_mutate(business_id, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;

				// Ensure caller is current owner
				ensure!(business.owner == who, Error::<T>::NotBusinessOwner);

				// Update owner
				let old_owner = business.owner.clone();
				business.owner = new_owner.clone();
				business.owner_user_index = new_owner_user_index;

				// Update roles
				BusinessMembers::<T>::insert(business_id, &new_owner, MemberRole::Owner);
				BusinessMembers::<T>::insert(business_id, &old_owner, MemberRole::Manager);

				// Emit event
				Self::deposit_event(Event::OwnershipTransferred {
					business_id,
					old_owner,
					new_owner,
				});

				Ok(())
			})
		}

		/// Verify a business (requires root/sudo)
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::verify_business())]
		pub fn verify_business(origin: OriginFor<T>, business_id: u32) -> DispatchResult {
			ensure_root(origin)?;

			// Update verification status
			Businesss::<T>::try_mutate(business_id, |maybe_business| -> DispatchResult {
				let business = maybe_business.as_mut().ok_or(Error::<T>::BusinessNotFound)?;
				business.is_verified = true;
				Ok(())
			})?;

			// Emit event
			Self::deposit_event(Event::BusinessVerified { business_id });

			Ok(())
		}
	}
}
