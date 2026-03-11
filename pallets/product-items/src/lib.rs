//! # Product Items Pallet
//!
//! A FRAME pallet for managing reusable product attribute definitions with units.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating product attribute definitions (e.g., "Weight", "Volume")
//! - Assigning units to attributes (e.g., "kg", "liter")
//! - Managing attribute lifecycle (create, update, deactivate)
//! - Business-scoped attribute definitions
//!
//! ## Purpose
//!
//! Product Items are reusable attribute templates that ensure consistent product data entry.
//! Instead of free-form product attributes, users define items like:
//! - Weight (kg)
//! - Volume (liter)
//! - Length (meter)
//! - Temperature (°C)
//!
//! These items can then be referenced when creating products with typed values.
//!
//! ## Blockchain Recovery
//!
//! All product items are stored on-chain to enable database recovery.
//! If the PostgreSQL database is lost, all product item definitions can be reconstructed
//! from blockchain data.

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

		/// Maximum length for item name (e.g., "Weight", "Volume")
		#[pallet::constant]
		type MaxItemNameLength: Get<u32>;

		/// Maximum length for unit (e.g., "kg", "liter")
		#[pallet::constant]
		type MaxUnitLength: Get<u32>;

		/// Maximum length for description
		#[pallet::constant]
		type MaxDescriptionLength: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Product Item definition (reusable attribute with unit)
	#[derive(CloneNoBound, Encode, Decode, Eq, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ProductItem<T: Config> {
		/// Sequential item ID within business scope
		pub item_id: u32,
		/// Business UUID reference (16 bytes)
		pub business_id: [u8; 16],
		/// Item name (e.g., "Weight", "Volume", "Length")
		pub name: BoundedVec<u8, T::MaxItemNameLength>,
		/// Unit of measurement (e.g., "kg", "liter", "meter")
		pub unit: BoundedVec<u8, T::MaxUnitLength>,
		/// Optional description for the item
		pub description: Option<BoundedVec<u8, T::MaxDescriptionLength>>,
		/// Active status (false = soft deleted)
		pub is_active: bool,
		/// Block number when created
		pub created_at: BlockNumberFor<T>,
		/// Block number when last updated
		pub updated_at: BlockNumberFor<T>,
	}

	/// Storage: Product Items by business_id and item_id
	/// Double map allows efficient querying of all items for a business
	#[pallet::storage]
	pub type ProductItems<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		[u8; 16],     // business_id
		Blake2_128Concat,
		u32,          // item_id
		ProductItem<T>,
	>;

	/// Storage: Item name to item_id lookup for uniqueness validation
	/// Ensures item names are unique within each business
	#[pallet::storage]
	pub type ProductItemNames<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		[u8; 16],     // business_id
		Blake2_128Concat,
		BoundedVec<u8, T::MaxItemNameLength>,  // name (lowercase)
		u32,          // item_id
	>;

	/// Storage: Next item ID counter per business
	/// Auto-increments to generate unique item IDs
	#[pallet::storage]
	pub type NextItemId<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 16],     // business_id
		u32,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Product item created
		ProductItemCreated {
			business_id: [u8; 16],
			item_id: u32,
			name: Vec<u8>,
			unit: Vec<u8>,
		},
		/// Product item updated
		ProductItemUpdated {
			business_id: [u8; 16],
			item_id: u32,
		},
		/// Product item deactivated (soft delete)
		ProductItemDeactivated {
			business_id: [u8; 16],
			item_id: u32,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Product item not found
		ProductItemNotFound,
		/// Product item name already exists in this business
		ProductItemNameAlreadyExists,
		/// Product item is used in products and cannot be deleted
		ProductItemInUse,
		/// User is not a member of this business
		NotBusinessMember,
		/// Invalid item name format (empty or too long)
		InvalidItemName,
		/// Invalid unit format (empty or too long)
		InvalidUnit,
		/// Item is already inactive
		ItemAlreadyInactive,
		/// Cannot modify inactive item
		CannotModifyInactiveItem,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new product item
		///
		/// # Parameters
		/// - `origin`: The caller (must be business member)
		/// - `business_id`: Business UUID (16 bytes)
		/// - `name`: Item name (e.g., "Weight", "Volume")
		/// - `unit`: Unit of measurement (e.g., "kg", "liter")
		/// - `description`: Optional description
		///
		/// # Errors
		/// - `NotBusinessMember`: Caller is not a business member
		/// - `InvalidItemName`: Name is empty or exceeds max length
		/// - `InvalidUnit`: Unit is empty or exceeds max length
		/// - `ProductItemNameAlreadyExists`: Name already exists in business
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(3))]
		pub fn create_product_item(
			origin: OriginFor<T>,
			business_id: [u8; 16],
			name: Vec<u8>,
			unit: Vec<u8>,
			description: Option<Vec<u8>>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// TODO: Verify caller is business member (integrate with business-management pallet)
			// For now, we allow any signed origin

			// Validate name
			ensure!(!name.is_empty(), Error::<T>::InvalidItemName);
			let bounded_name: BoundedVec<u8, T::MaxItemNameLength> =
				name.clone().try_into().map_err(|_| Error::<T>::InvalidItemName)?;

			// Validate unit
			ensure!(!unit.is_empty(), Error::<T>::InvalidUnit);
			let bounded_unit: BoundedVec<u8, T::MaxUnitLength> =
				unit.clone().try_into().map_err(|_| Error::<T>::InvalidUnit)?;

			// Validate description if provided
			let bounded_description = if let Some(desc) = description {
				let bounded: BoundedVec<u8, T::MaxDescriptionLength> =
					desc.try_into().map_err(|_| Error::<T>::InvalidItemName)?;
				Some(bounded)
			} else {
				None
			};

			// Check name uniqueness (case-insensitive)
			let lowercase_name: Vec<u8> = name.iter().map(|c| c.to_ascii_lowercase()).collect();
			let bounded_lowercase_name: BoundedVec<u8, T::MaxItemNameLength> =
				lowercase_name.try_into().map_err(|_| Error::<T>::InvalidItemName)?;

			ensure!(
				!ProductItemNames::<T>::contains_key(&business_id, &bounded_lowercase_name),
				Error::<T>::ProductItemNameAlreadyExists
			);

			// Generate new item ID
			let item_id = NextItemId::<T>::get(&business_id);
			let next_id = item_id.checked_add(1).ok_or(Error::<T>::ProductItemNotFound)?;
			NextItemId::<T>::insert(&business_id, next_id);

			// Get current block number
			let current_block = frame_system::Pallet::<T>::block_number();

			// Create product item
			let product_item = ProductItem {
				item_id,
				business_id,
				name: bounded_name.clone(),
				unit: bounded_unit.clone(),
				description: bounded_description,
				is_active: true,
				created_at: current_block,
				updated_at: current_block,
			};

			// Store product item
			ProductItems::<T>::insert(&business_id, item_id, product_item);

			// Store name lookup for uniqueness
			ProductItemNames::<T>::insert(&business_id, &bounded_lowercase_name, item_id);

			// Emit event
			Self::deposit_event(Event::ProductItemCreated {
				business_id,
				item_id,
				name,
				unit,
			});

			Ok(())
		}

		/// Update an existing product item
		///
		/// # Parameters
		/// - `origin`: The caller (must be business member)
		/// - `business_id`: Business UUID
		/// - `item_id`: Item ID to update
		/// - `name`: New name (optional)
		/// - `unit`: New unit (optional)
		/// - `description`: New description (optional)
		/// - `is_active`: New active status (optional)
		///
		/// # Errors
		/// - `ProductItemNotFound`: Item doesn't exist
		/// - `CannotModifyInactiveItem`: Cannot update inactive item
		/// - `ProductItemNameAlreadyExists`: New name conflicts with existing item
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
		pub fn update_product_item(
			origin: OriginFor<T>,
			business_id: [u8; 16],
			item_id: u32,
			name: Option<Vec<u8>>,
			unit: Option<Vec<u8>>,
			description: Option<Option<Vec<u8>>>,
			is_active: Option<bool>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// TODO: Verify caller is business member

			// Get existing item
			let mut item = ProductItems::<T>::get(&business_id, item_id)
				.ok_or(Error::<T>::ProductItemNotFound)?;

			// Cannot modify inactive items (except to reactivate)
			if !item.is_active && is_active != Some(true) {
				return Err(Error::<T>::CannotModifyInactiveItem.into());
			}

			// Update name if provided
			if let Some(new_name) = name {
				ensure!(!new_name.is_empty(), Error::<T>::InvalidItemName);

				let bounded_name: BoundedVec<u8, T::MaxItemNameLength> =
					new_name.clone().try_into().map_err(|_| Error::<T>::InvalidItemName)?;

				// Check uniqueness if name changed
				let lowercase_name: Vec<u8> = new_name.iter().map(|c| c.to_ascii_lowercase()).collect();

				// Only check uniqueness if name actually changed
				let old_lowercase: Vec<u8> = item.name.iter().map(|c| c.to_ascii_lowercase()).collect();
				if lowercase_name != old_lowercase {
					let bounded_lowercase_name: BoundedVec<u8, T::MaxItemNameLength> =
						lowercase_name.try_into().map_err(|_| Error::<T>::InvalidItemName)?;
					ensure!(
						!ProductItemNames::<T>::contains_key(&business_id, &bounded_lowercase_name),
						Error::<T>::ProductItemNameAlreadyExists
					);

					// Remove old name lookup
					let old_bounded_lowercase: BoundedVec<u8, T::MaxItemNameLength> =
						old_lowercase.try_into().map_err(|_| Error::<T>::InvalidItemName)?;
					ProductItemNames::<T>::remove(&business_id, &old_bounded_lowercase);

					// Insert new name lookup
					ProductItemNames::<T>::insert(&business_id, &bounded_lowercase_name, item_id);
				}

				item.name = bounded_name;
			}

			// Update unit if provided
			if let Some(new_unit) = unit {
				ensure!(!new_unit.is_empty(), Error::<T>::InvalidUnit);
				let bounded_unit: BoundedVec<u8, T::MaxUnitLength> =
					new_unit.try_into().map_err(|_| Error::<T>::InvalidUnit)?;
				item.unit = bounded_unit;
			}

			// Update description if provided (Some(None) means remove description)
			if let Some(new_desc) = description {
				item.description = if let Some(desc) = new_desc {
					let bounded: BoundedVec<u8, T::MaxDescriptionLength> =
						desc.try_into().map_err(|_| Error::<T>::InvalidItemName)?;
					Some(bounded)
				} else {
					None
				};
			}

			// Update active status if provided
			if let Some(active) = is_active {
				item.is_active = active;

				if !active {
					// Emit deactivation event
					Self::deposit_event(Event::ProductItemDeactivated {
						business_id,
						item_id,
					});
				}
			}

			// Update timestamp
			item.updated_at = frame_system::Pallet::<T>::block_number();

			// Save updated item
			ProductItems::<T>::insert(&business_id, item_id, item);

			// Emit update event
			Self::deposit_event(Event::ProductItemUpdated {
				business_id,
				item_id,
			});

			Ok(())
		}

		/// Deactivate a product item (soft delete)
		///
		/// # Parameters
		/// - `origin`: The caller (must be business member)
		/// - `business_id`: Business UUID
		/// - `item_id`: Item ID to deactivate
		///
		/// # Errors
		/// - `ProductItemNotFound`: Item doesn't exist
		/// - `ItemAlreadyInactive`: Item is already inactive
		/// - `ProductItemInUse`: Item is referenced by products (check in product-management)
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn deactivate_product_item(
			origin: OriginFor<T>,
			business_id: [u8; 16],
			item_id: u32,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// TODO: Verify caller is business member
			// TODO: Check if item is used in any products (integrate with product-management)

			// Get existing item
			let mut item = ProductItems::<T>::get(&business_id, item_id)
				.ok_or(Error::<T>::ProductItemNotFound)?;

			ensure!(item.is_active, Error::<T>::ItemAlreadyInactive);

			// Deactivate
			item.is_active = false;
			item.updated_at = frame_system::Pallet::<T>::block_number();

			// Save
			ProductItems::<T>::insert(&business_id, item_id, item);

			// Emit event
			Self::deposit_event(Event::ProductItemDeactivated {
				business_id,
				item_id,
			});

			Ok(())
		}
	}
}
