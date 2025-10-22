//! # Role Permissions Pallet
//!
//! A FRAME pallet for role-based access control in a supply chain system.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating and managing roles with specific permissions
//! - Assigning roles to users within companies
//! - Permission checking for access control
//! - System-defined roles (Owner, Manager, Warehouse, Transport, Supplier)
//!
//! ## Dispatchable Functions
//!
//! - `create_role` - Create a new custom role for a company
//! - `assign_role` - Assign a role to a user within a company
//! - `revoke_role` - Revoke a user's role within a company
//! - `update_role_permissions` - Update permissions for a role
//!
//! ## Helper Functions
//!
//! - `check_permission` - Check if a user has a specific permission in a company

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
	use scale_info::prelude::{vec, vec::Vec};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Maximum length of role name
		#[pallet::constant]
		type MaxRoleNameLength: Get<u32>;

		/// Maximum number of permissions per role
		#[pallet::constant]
		type MaxPermissions: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Permission types available in the system
	#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Permission {
		/// Create new products
		CreateProduct,
		/// Update existing products
		UpdateProduct,
		/// Delete products
		DeleteProduct,
		/// View product information
		ViewProduct,
		/// Manage users
		ManageUsers,
		/// Manage roles and permissions
		ManageRoles,
		/// View reports and analytics
		ViewReports,
		/// Create shipments
		CreateShipment,
		/// Update shipment status
		UpdateShipment,
		/// Manage company settings
		ManageCompany,
	}

	/// Role information
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Role<T: Config> {
		/// Unique role identifier
		pub role_id: u32,
		/// Role name
		pub name: BoundedVec<u8, T::MaxRoleNameLength>,
		/// List of permissions for this role
		pub permissions: BoundedVec<Permission, T::MaxPermissions>,
		/// Company this role belongs to (None for system roles)
		pub company_id: Option<u32>,
		/// Whether this is a system-defined role
		pub is_system_role: bool,
		/// Creation timestamp
		pub created_at: BlockNumberFor<T>,
	}

	/// Role assignment information
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct RoleAssignment<T: Config> {
		/// User account ID
		pub user: T::AccountId,
		/// Assigned role ID
		pub role_id: u32,
		/// Company ID
		pub company_id: u32,
		/// Assignment timestamp
		pub assigned_at: BlockNumberFor<T>,
	}

	/// Storage: Roles indexed by role ID
	#[pallet::storage]
	#[pallet::getter(fn roles)]
	pub type Roles<T: Config> = StorageMap<_, Blake2_128Concat, u32, Role<T>>;

	/// Storage: User roles - maps (user, company_id) to role_id
	#[pallet::storage]
	#[pallet::getter(fn user_roles)]
	pub type UserRoles<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u32, // company_id
		u32, // role_id
	>;

	/// Storage: Company roles - maps (company_id, role_id) to ()
	#[pallet::storage]
	#[pallet::getter(fn company_roles)]
	pub type CompanyRoles<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32, // company_id
		Blake2_128Concat,
		u32, // role_id
		(),
	>;

	/// Storage: Next available role ID
	#[pallet::storage]
	#[pallet::getter(fn next_role_id)]
	pub type NextRoleId<T> = StorageValue<_, u32, ValueQuery>;

	/// Storage: System roles - maps role_id to ()
	#[pallet::storage]
	#[pallet::getter(fn system_roles)]
	pub type SystemRoles<T> = StorageMap<_, Blake2_128Concat, u32, ()>;

	/// Genesis configuration for the pallet
	#[pallet::genesis_config]
	#[derive(frame::prelude::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		#[serde(skip)]
		pub _phantom: core::marker::PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// Initialize system roles
			Pallet::<T>::initialize_system_roles();
		}
	}

	/// Events for the pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Role created
		RoleCreated { role_id: u32, company_id: Option<u32>, name: Vec<u8> },
		/// Role assigned to user
		RoleAssigned { user: T::AccountId, role_id: u32, company_id: u32 },
		/// Role revoked from user
		RoleRevoked { user: T::AccountId, company_id: u32 },
		/// Role permissions updated
		PermissionsUpdated { role_id: u32 },
	}

	/// Errors for the pallet
	#[pallet::error]
	pub enum Error<T> {
		/// Role not found
		RoleNotFound,
		/// User already has a role in this company
		RoleAlreadyAssigned,
		/// User has no role in this company
		NoRoleAssigned,
		/// Cannot modify system role
		CannotModifySystemRole,
		/// Role name too long
		RoleNameTooLong,
		/// Too many permissions
		TooManyPermissions,
		/// Not authorized to perform this action
		NotAuthorized,
		/// Role does not belong to this company
		RoleNotInCompany,
		/// Numeric overflow
		Overflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Dispatchable functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new role for a company
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_role())]
		pub fn create_role(
			origin: OriginFor<T>,
			company_id: u32,
			name: Vec<u8>,
			permissions: Vec<Permission>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if user has permission to create roles in this company
			ensure!(
				Self::check_permission(&who, company_id, Permission::ManageRoles),
				Error::<T>::NotAuthorized
			);

			// Convert to bounded vecs
			let bounded_name: BoundedVec<u8, T::MaxRoleNameLength> =
				name.clone().try_into().map_err(|_| Error::<T>::RoleNameTooLong)?;
			let bounded_permissions: BoundedVec<Permission, T::MaxPermissions> =
				permissions.try_into().map_err(|_| Error::<T>::TooManyPermissions)?;

			// Get next role ID
			let role_id = NextRoleId::<T>::get();
			let next_id = role_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

			let now = frame_system::Pallet::<T>::block_number();

			// Create role
			let role = Role {
				role_id,
				name: bounded_name,
				permissions: bounded_permissions,
				company_id: Some(company_id),
				is_system_role: false,
				created_at: now,
			};

			// Store role
			Roles::<T>::insert(role_id, role);
			CompanyRoles::<T>::insert(company_id, role_id, ());
			NextRoleId::<T>::put(next_id);

			// Emit event
			Self::deposit_event(Event::RoleCreated {
				role_id,
				company_id: Some(company_id),
				name,
			});

			Ok(())
		}

		/// Assign a role to a user within a company
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::assign_role())]
		pub fn assign_role(
			origin: OriginFor<T>,
			user: T::AccountId,
			company_id: u32,
			role_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if caller has permission to manage users
			ensure!(
				Self::check_permission(&who, company_id, Permission::ManageUsers),
				Error::<T>::NotAuthorized
			);

			// Verify role exists
			let role = Roles::<T>::get(role_id).ok_or(Error::<T>::RoleNotFound)?;

			// Verify role belongs to this company (or is a system role)
			if let Some(role_company_id) = role.company_id {
				ensure!(role_company_id == company_id, Error::<T>::RoleNotInCompany);
			}

			// Check if user already has a role in this company
			ensure!(
				!UserRoles::<T>::contains_key(&user, company_id),
				Error::<T>::RoleAlreadyAssigned
			);

			// Assign role
			UserRoles::<T>::insert(&user, company_id, role_id);

			// Emit event
			Self::deposit_event(Event::RoleAssigned {
				user,
				role_id,
				company_id,
			});

			Ok(())
		}

		/// Revoke a user's role within a company
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::revoke_role())]
		pub fn revoke_role(
			origin: OriginFor<T>,
			user: T::AccountId,
			company_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if caller has permission to manage users
			ensure!(
				Self::check_permission(&who, company_id, Permission::ManageUsers),
				Error::<T>::NotAuthorized
			);

			// Verify user has a role in this company
			ensure!(
				UserRoles::<T>::contains_key(&user, company_id),
				Error::<T>::NoRoleAssigned
			);

			// Remove role assignment
			UserRoles::<T>::remove(&user, company_id);

			// Emit event
			Self::deposit_event(Event::RoleRevoked {
				user,
				company_id,
			});

			Ok(())
		}

		/// Update permissions for a role
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_role_permissions())]
		pub fn update_role_permissions(
			origin: OriginFor<T>,
			role_id: u32,
			permissions: Vec<Permission>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get role
			let role = Roles::<T>::get(role_id).ok_or(Error::<T>::RoleNotFound)?;

			// Cannot modify system roles
			ensure!(!role.is_system_role, Error::<T>::CannotModifySystemRole);

			// If role belongs to a company, check permissions
			if let Some(company_id) = role.company_id {
				ensure!(
					Self::check_permission(&who, company_id, Permission::ManageRoles),
					Error::<T>::NotAuthorized
				);
			}

			// Convert to bounded vec
			let bounded_permissions: BoundedVec<Permission, T::MaxPermissions> =
				permissions.try_into().map_err(|_| Error::<T>::TooManyPermissions)?;

			// Update role permissions
			Roles::<T>::mutate(role_id, |maybe_role| {
				if let Some(r) = maybe_role {
					r.permissions = bounded_permissions;
				}
			});

			// Emit event
			Self::deposit_event(Event::PermissionsUpdated { role_id });

			Ok(())
		}
	}

	/// Helper functions
	impl<T: Config> Pallet<T> {
		/// Check if a user has a specific permission within a company
		pub fn check_permission(
			user: &T::AccountId,
			company_id: u32,
			permission: Permission,
		) -> bool {
			// Get user's role in the company
			if let Some(role_id) = UserRoles::<T>::get(user, company_id) {
				// Get role details
				if let Some(role) = Roles::<T>::get(role_id) {
					// Check if role has the required permission
					return role.permissions.contains(&permission);
				}
			}
			false
		}

		/// Initialize system roles
		pub fn initialize_system_roles() {
			let mut role_id = 0u32;

			// Role 1: Owner - all permissions
			let owner_permissions = vec![
				Permission::CreateProduct,
				Permission::UpdateProduct,
				Permission::DeleteProduct,
				Permission::ViewProduct,
				Permission::ManageUsers,
				Permission::ManageRoles,
				Permission::ViewReports,
				Permission::CreateShipment,
				Permission::UpdateShipment,
				Permission::ManageCompany,
			];
			Self::create_system_role(role_id, b"Owner".to_vec(), owner_permissions);
			role_id += 1;

			// Role 2: Manager - most permissions except ManageCompany
			let manager_permissions = vec![
				Permission::CreateProduct,
				Permission::UpdateProduct,
				Permission::DeleteProduct,
				Permission::ViewProduct,
				Permission::ManageUsers,
				Permission::ManageRoles,
				Permission::ViewReports,
				Permission::CreateShipment,
				Permission::UpdateShipment,
			];
			Self::create_system_role(role_id, b"Manager".to_vec(), manager_permissions);
			role_id += 1;

			// Role 3: Warehouse - product and shipment management
			let warehouse_permissions = vec![
				Permission::CreateProduct,
				Permission::UpdateProduct,
				Permission::ViewProduct,
				Permission::CreateShipment,
				Permission::UpdateShipment,
			];
			Self::create_system_role(role_id, b"Warehouse".to_vec(), warehouse_permissions);
			role_id += 1;

			// Role 4: Transport - view and update shipments
			let transport_permissions = vec![
				Permission::ViewProduct,
				Permission::UpdateShipment,
				Permission::ViewReports,
			];
			Self::create_system_role(role_id, b"Transport".to_vec(), transport_permissions);
			role_id += 1;

			// Role 5: Supplier - create and view products
			let supplier_permissions = vec![
				Permission::CreateProduct,
				Permission::ViewProduct,
				Permission::ViewReports,
			];
			Self::create_system_role(role_id, b"Supplier".to_vec(), supplier_permissions);
			role_id += 1;

			// Set next role ID
			NextRoleId::<T>::put(role_id);
		}

		/// Create a system role (internal helper)
		fn create_system_role(role_id: u32, name: Vec<u8>, permissions: Vec<Permission>) {
			let bounded_name: BoundedVec<u8, T::MaxRoleNameLength> =
				name.try_into().expect("System role name too long");
			let bounded_permissions: BoundedVec<Permission, T::MaxPermissions> =
				permissions.try_into().expect("Too many permissions in system role");

			let role = Role {
				role_id,
				name: bounded_name,
				permissions: bounded_permissions,
				company_id: None,
				is_system_role: true,
				created_at: BlockNumberFor::<T>::default(),
			};

			Roles::<T>::insert(role_id, role);
			SystemRoles::<T>::insert(role_id, ());
		}

		/// Get all permissions for a user in a company
		pub fn get_user_permissions(
			user: &T::AccountId,
			company_id: u32,
		) -> Option<Vec<Permission>> {
			if let Some(role_id) = UserRoles::<T>::get(user, company_id) {
				if let Some(role) = Roles::<T>::get(role_id) {
					return Some(role.permissions.to_vec());
				}
			}
			None
		}

		/// Check if a role is a system role
		pub fn is_system_role(role_id: u32) -> bool {
			SystemRoles::<T>::contains_key(role_id)
		}
	}
}
