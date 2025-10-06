//! # Company Management Pallet
//!
//! A FRAME pallet for managing companies in a supply chain system.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating and managing companies
//! - Inviting team members via email/wallet
//! - Company settings and configuration
//! - Ownership transfer between users
//! - Company verification and status management

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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxCompanyNameLength: Get<u32>;

		#[pallet::constant]
		type MaxMembers: Get<u32>;

		#[pallet::constant]
		type MaxPendingInvites: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Company information
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Company<T: Config> {
		pub name: BoundedVec<u8, T::MaxCompanyNameLength>,
		pub owner: T::AccountId,
		pub created_at: BlockNumberFor<T>,
		pub is_verified: bool,
		pub member_count: u32,
	}

	/// Company member details
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

	/// Company invitation
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Invitation<T: Config> {
		pub company_id: u32,
		pub invitee: T::AccountId,
		pub role: MemberRole,
		pub status: InviteStatus,
		pub invited_at: BlockNumberFor<T>,
	}

	/// Storage: Companies indexed by ID
	#[pallet::storage]
	#[pallet::getter(fn companies)]
	pub type Companies<T: Config> = StorageMap<_, Blake2_128Concat, u32, Company<T>>;

	/// Storage: Company members
	#[pallet::storage]
	#[pallet::getter(fn company_members)]
	pub type CompanyMembers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, u32,  // company_id
		Blake2_128Concat, T::AccountId,  // account
		MemberRole,
	>;

	/// Storage: User to company mapping
	#[pallet::storage]
	#[pallet::getter(fn user_company)]
	pub type UserCompany<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

	/// Storage: Pending invitations
	#[pallet::storage]
	#[pallet::getter(fn invitations)]
	pub type Invitations<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Invitation<T>>;

	/// Storage: Next company ID
	#[pallet::storage]
	#[pallet::getter(fn next_company_id)]
	pub type NextCompanyId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CompanyCreated { company_id: u32, owner: T::AccountId, name: Vec<u8> },
		MemberInvited { company_id: u32, invitee: T::AccountId, role: MemberRole },
		InvitationAccepted { company_id: u32, member: T::AccountId },
		InvitationRejected { company_id: u32, invitee: T::AccountId },
		MemberRemoved { company_id: u32, member: T::AccountId },
		OwnershipTransferred { company_id: u32, old_owner: T::AccountId, new_owner: T::AccountId },
		CompanyVerified { company_id: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
		CompanyNotFound,
		CompanyAlreadyExists,
		NotCompanyOwner,
		NotCompanyMember,
		AlreadyMember,
		InvitationNotFound,
		InvalidInvitation,
		MaxMembersReached,
		NameTooLong,
		NotAuthorized,
		CannotRemoveOwner,
		CannotTransferToNonMember,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new company
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_company())]
		pub fn create_company(
			origin: OriginFor<T>,
			name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure user doesn't already have a company
			ensure!(!UserCompany::<T>::contains_key(&who), Error::<T>::CompanyAlreadyExists);

			let bounded_name: BoundedVec<u8, T::MaxCompanyNameLength> =
				name.clone().try_into().map_err(|_| Error::<T>::NameTooLong)?;

			let company_id = NextCompanyId::<T>::get();
			let now = frame_system::Pallet::<T>::block_number();

			let company = Company {
				name: bounded_name,
				owner: who.clone(),
				created_at: now,
				is_verified: false,
				member_count: 1,
			};

			Companies::<T>::insert(company_id, company);
			CompanyMembers::<T>::insert(company_id, &who, MemberRole::Owner);
			UserCompany::<T>::insert(&who, company_id);
			NextCompanyId::<T>::put(company_id.saturating_add(1));

			Self::deposit_event(Event::CompanyCreated { company_id, owner: who, name });

			Ok(())
		}

		/// Invite a member to the company
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::invite_member())]
		pub fn invite_member(
			origin: OriginFor<T>,
			company_id: u32,
			invitee: T::AccountId,
			role: MemberRole,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let company = Companies::<T>::get(company_id).ok_or(Error::<T>::CompanyNotFound)?;

			// Only owner or manager can invite
			let member_role = CompanyMembers::<T>::get(company_id, &who)
				.ok_or(Error::<T>::NotCompanyMember)?;

			ensure!(
				matches!(member_role, MemberRole::Owner | MemberRole::Manager),
				Error::<T>::NotAuthorized
			);

			// Check if invitee is already a member
			ensure!(!UserCompany::<T>::contains_key(&invitee), Error::<T>::AlreadyMember);

			// Check max members
			ensure!(company.member_count < T::MaxMembers::get(), Error::<T>::MaxMembersReached);

			let invitation = Invitation {
				company_id,
				invitee: invitee.clone(),
				role: role.clone(),
				status: InviteStatus::Pending,
				invited_at: frame_system::Pallet::<T>::block_number(),
			};

			Invitations::<T>::insert(&invitee, invitation);

			Self::deposit_event(Event::MemberInvited { company_id, invitee, role });

			Ok(())
		}

		/// Accept an invitation
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::accept_invitation())]
		pub fn accept_invitation(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut invitation = Invitations::<T>::get(&who)
				.ok_or(Error::<T>::InvitationNotFound)?;

			ensure!(invitation.status == InviteStatus::Pending, Error::<T>::InvalidInvitation);

			invitation.status = InviteStatus::Accepted;

			Companies::<T>::try_mutate(invitation.company_id, |maybe_company| -> DispatchResult {
				let company = maybe_company.as_mut().ok_or(Error::<T>::CompanyNotFound)?;
				company.member_count = company.member_count.saturating_add(1);
				Ok(())
			})?;

			CompanyMembers::<T>::insert(invitation.company_id, &who, invitation.role);
			UserCompany::<T>::insert(&who, invitation.company_id);
			Invitations::<T>::remove(&who);

			Self::deposit_event(Event::InvitationAccepted {
				company_id: invitation.company_id,
				member: who
			});

			Ok(())
		}

		/// Reject an invitation
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::reject_invitation())]
		pub fn reject_invitation(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let invitation = Invitations::<T>::get(&who)
				.ok_or(Error::<T>::InvitationNotFound)?;

			Invitations::<T>::remove(&who);

			Self::deposit_event(Event::InvitationRejected {
				company_id: invitation.company_id,
				invitee: who
			});

			Ok(())
		}

		/// Remove a member from company
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_member())]
		pub fn remove_member(
			origin: OriginFor<T>,
			company_id: u32,
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let company = Companies::<T>::get(company_id).ok_or(Error::<T>::CompanyNotFound)?;
			ensure!(company.owner == who, Error::<T>::NotCompanyOwner);

			let member_role = CompanyMembers::<T>::get(company_id, &member)
				.ok_or(Error::<T>::NotCompanyMember)?;

			ensure!(member_role != MemberRole::Owner, Error::<T>::CannotRemoveOwner);

			CompanyMembers::<T>::remove(company_id, &member);
			UserCompany::<T>::remove(&member);

			Companies::<T>::try_mutate(company_id, |maybe_company| -> DispatchResult {
				let company = maybe_company.as_mut().ok_or(Error::<T>::CompanyNotFound)?;
				company.member_count = company.member_count.saturating_sub(1);
				Ok(())
			})?;

			Self::deposit_event(Event::MemberRemoved { company_id, member });

			Ok(())
		}

		/// Transfer company ownership
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::transfer_ownership())]
		pub fn transfer_ownership(
			origin: OriginFor<T>,
			company_id: u32,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure new owner is a member
			ensure!(
				CompanyMembers::<T>::contains_key(company_id, &new_owner),
				Error::<T>::CannotTransferToNonMember
			);

			Companies::<T>::try_mutate(company_id, |maybe_company| -> DispatchResult {
				let company = maybe_company.as_mut().ok_or(Error::<T>::CompanyNotFound)?;
				ensure!(company.owner == who, Error::<T>::NotCompanyOwner);

				company.owner = new_owner.clone();
				Ok(())
			})?;

			// Update roles
			CompanyMembers::<T>::insert(company_id, &new_owner, MemberRole::Owner);
			CompanyMembers::<T>::insert(company_id, &who, MemberRole::Manager);

			Self::deposit_event(Event::OwnershipTransferred {
				company_id,
				old_owner: who,
				new_owner
			});

			Ok(())
		}

		/// Verify a company (root only)
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::verify_company())]
		pub fn verify_company(
			origin: OriginFor<T>,
			company_id: u32,
		) -> DispatchResult {
			ensure_root(origin)?;

			Companies::<T>::try_mutate(company_id, |maybe_company| -> DispatchResult {
				let company = maybe_company.as_mut().ok_or(Error::<T>::CompanyNotFound)?;
				company.is_verified = true;
				Ok(())
			})?;

			Self::deposit_event(Event::CompanyVerified { company_id });

			Ok(())
		}
	}
}
