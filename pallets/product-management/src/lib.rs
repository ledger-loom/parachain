//! # Product Management Pallet
//!
//! A FRAME pallet for managing products in a supply chain system.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Adding products with basic info (name, description, category)
//! - Custom attributes for products
//! - Product categories and organization
//! - Product updates and modifications
//! - Batch management

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
	use frame::deps::codec::{Decode, Encode, MaxEncodedLen, DecodeWithMemTracking};
	use scale_info::prelude::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxProductNameLength: Get<u32>;

		#[pallet::constant]
		type MaxCategoryLength: Get<u32>;

		#[pallet::constant]
		type MaxAttributes: Get<u32>;

		#[pallet::constant]
		type MaxAttributeKeyLength: Get<u32>;

		#[pallet::constant]
		type MaxAttributeValueLength: Get<u32>;

		/// Maximum length for encrypted data
		#[pallet::constant]
		type MaxEncryptedDataLength: Get<u32>;

		/// Maximum number of authorized roles
		#[pallet::constant]
		type MaxAuthorizedRoles: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Product attribute
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ProductAttribute<T: Config> {
		pub key: BoundedVec<u8, T::MaxAttributeKeyLength>,
		pub value: BoundedVec<u8, T::MaxAttributeValueLength>,
	}

	/// Product status
	#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum ProductStatus {
		Active,
		Inactive,
		Discontinued,
		Draft,
	}

	/// Data visibility level
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum VisibilityLevel {
		Public,
		Company,
		Management,
		Restricted,
		Private,
	}

	/// Product information (with encryption support)
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Product<T: Config> {
		// Public metadata (unencrypted)
		pub product_id: u32,
		pub company_id: u32,
		pub status: ProductStatus,
		pub category: BoundedVec<u8, T::MaxCategoryLength>,
		pub created_at: BlockNumberFor<T>,
		pub updated_at: BlockNumberFor<T>,

		// Encrypted sensitive data
		pub encrypted_name: BoundedVec<u8, T::MaxEncryptedDataLength>,
		pub encrypted_attributes: BoundedVec<u8, T::MaxEncryptedDataLength>,

		// Encryption metadata
		pub data_hash: [u8; 32],
		pub encryption_key_id: Option<BoundedVec<u8, T::MaxProductNameLength>>,
		pub is_encrypted: bool,

		// Access control
		pub visibility: VisibilityLevel,
		pub authorized_roles: BoundedVec<u32, T::MaxAuthorizedRoles>,
	}

	/// Storage: Products indexed by ID
	#[pallet::storage]
	#[pallet::getter(fn products)]
	pub type Products<T: Config> = StorageMap<_, Blake2_128Concat, u32, Product<T>>;

	/// Storage: Company products
	#[pallet::storage]
	#[pallet::getter(fn company_products)]
	pub type CompanyProducts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, u32,  // company_id
		Blake2_128Concat, u32,  // product_id
		(),
	>;

	/// Storage: Next product ID
	#[pallet::storage]
	#[pallet::getter(fn next_product_id)]
	pub type NextProductId<T> = StorageValue<_, u32, ValueQuery>;

	/// Storage: Product categories
	#[pallet::storage]
	#[pallet::getter(fn categories)]
	pub type Categories<T: Config> = StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxCategoryLength>, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProductCreated { product_id: u32, company_id: u32, name: Vec<u8> },
		ProductCreatedEncrypted { product_id: u32, company_id: u32, data_hash: [u8; 32] },
		ProductUpdated { product_id: u32 },
		ProductStatusChanged { product_id: u32, status: ProductStatus },
		AttributeAdded { product_id: u32, key: Vec<u8> },
		AttributeUpdated { product_id: u32, key: Vec<u8> },
		CategoryCreated { category: Vec<u8> },
		ProductAccessGranted { product_id: u32, accessor: T::AccountId },
		VisibilityUpdated { product_id: u32, new_visibility: VisibilityLevel },
	}

	#[pallet::error]
	pub enum Error<T> {
		ProductNotFound,
		NotProductOwner,
		NotAuthorized,
		ProductNameTooLong,
		CategoryTooLong,
		AttributeKeyTooLong,
		AttributeValueTooLong,
		MaxAttributesReached,
		AttributeNotFound,
		DuplicateAttribute,
		EncryptedDataTooLong,
		TooManyAuthorizedRoles,
		InvalidEncryptionKey,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new product
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_product())]
		pub fn create_product(
			origin: OriginFor<T>,
			company_id: u32,
			name: Vec<u8>,
			category: Vec<u8>,
			attributes: Vec<(Vec<u8>, Vec<u8>)>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let bounded_name: BoundedVec<u8, T::MaxProductNameLength> =
				name.clone().try_into().map_err(|_| Error::<T>::ProductNameTooLong)?;

			let bounded_category: BoundedVec<u8, T::MaxCategoryLength> =
				category.clone().try_into().map_err(|_| Error::<T>::CategoryTooLong)?;

			let mut bounded_attrs = BoundedVec::<ProductAttribute<T>, T::MaxAttributes>::default();
			for (key, value) in attributes.iter() {
				let attr_key: BoundedVec<u8, T::MaxAttributeKeyLength> =
					key.clone().try_into().map_err(|_| Error::<T>::AttributeKeyTooLong)?;
				let attr_value: BoundedVec<u8, T::MaxAttributeValueLength> =
					value.clone().try_into().map_err(|_| Error::<T>::AttributeValueTooLong)?;

				bounded_attrs.try_push(ProductAttribute {
					key: attr_key,
					value: attr_value,
				}).map_err(|_| Error::<T>::MaxAttributesReached)?;
			}

			let product_id = NextProductId::<T>::get();
			let now = frame_system::Pallet::<T>::block_number();

			let product = Product {
				name: bounded_name,
				category: bounded_category.clone(),
				company_id,
				status: ProductStatus::Active,
				created_at: now,
				updated_at: now,
				attributes: bounded_attrs,
			};

			Products::<T>::insert(product_id, product);
			CompanyProducts::<T>::insert(company_id, product_id, ());
			NextProductId::<T>::put(product_id.saturating_add(1));

			// Track category
			Categories::<T>::mutate(&bounded_category, |count| {
				*count = Some(count.map_or(1, |c| c.saturating_add(1)));
			});

			Self::deposit_event(Event::ProductCreated { product_id, company_id, name });

			Ok(())
		}

		/// Update product status
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_product_status())]
		pub fn update_product_status(
			origin: OriginFor<T>,
			product_id: u32,
			status: ProductStatus,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
				let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;
				product.status = status.clone();
				product.updated_at = frame_system::Pallet::<T>::block_number();
				Ok(())
			})?;

			Self::deposit_event(Event::ProductStatusChanged { product_id, status });

			Ok(())
		}

		/// Add attribute to product
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::add_attribute())]
		pub fn add_attribute(
			origin: OriginFor<T>,
			product_id: u32,
			key: Vec<u8>,
			value: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let attr_key: BoundedVec<u8, T::MaxAttributeKeyLength> =
				key.clone().try_into().map_err(|_| Error::<T>::AttributeKeyTooLong)?;
			let attr_value: BoundedVec<u8, T::MaxAttributeValueLength> =
				value.try_into().map_err(|_| Error::<T>::AttributeValueTooLong)?;

			Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
				let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;

				// Check if attribute already exists
				ensure!(
					!product.attributes.iter().any(|a| a.key == attr_key),
					Error::<T>::DuplicateAttribute
				);

				product.attributes.try_push(ProductAttribute {
					key: attr_key,
					value: attr_value,
				}).map_err(|_| Error::<T>::MaxAttributesReached)?;

				product.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::AttributeAdded { product_id, key });

			Ok(())
		}

		/// Update attribute value
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_attribute())]
		pub fn update_attribute(
			origin: OriginFor<T>,
			product_id: u32,
			key: Vec<u8>,
			value: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let attr_key: BoundedVec<u8, T::MaxAttributeKeyLength> =
				key.clone().try_into().map_err(|_| Error::<T>::AttributeKeyTooLong)?;
			let attr_value: BoundedVec<u8, T::MaxAttributeValueLength> =
				value.try_into().map_err(|_| Error::<T>::AttributeValueTooLong)?;

			Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
				let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;

				let attribute = product.attributes.iter_mut()
					.find(|a| a.key == attr_key)
					.ok_or(Error::<T>::AttributeNotFound)?;

				attribute.value = attr_value;
				product.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::AttributeUpdated { product_id, key });

			Ok(())
		}

		/// Create encrypted product (sensitive data stored as ciphertext)
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn create_encrypted_product(
			origin: OriginFor<T>,
			company_id: u32,
			encrypted_name: Vec<u8>,
			encrypted_attributes: Vec<u8>,
			category: Vec<u8>,
			data_hash: [u8; 32],
			encryption_key_id: Vec<u8>,
			visibility: VisibilityLevel,
			authorized_roles: Vec<u32>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let bounded_encrypted_name: BoundedVec<u8, T::MaxEncryptedDataLength> =
				encrypted_name.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			let bounded_encrypted_attrs: BoundedVec<u8, T::MaxEncryptedDataLength> =
				encrypted_attributes.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			let bounded_category: BoundedVec<u8, T::MaxCategoryLength> =
				category.try_into().map_err(|_| Error::<T>::CategoryTooLong)?;

			let bounded_key_id: BoundedVec<u8, T::MaxProductNameLength> =
				encryption_key_id.try_into().map_err(|_| Error::<T>::InvalidEncryptionKey)?;

			let bounded_roles: BoundedVec<u32, T::MaxAuthorizedRoles> =
				authorized_roles.try_into().map_err(|_| Error::<T>::TooManyAuthorizedRoles)?;

			let product_id = NextProductId::<T>::get();
			let now = frame_system::Pallet::<T>::block_number();

			let product = Product {
				product_id,
				company_id,
				status: ProductStatus::Active,
				category: bounded_category.clone(),
				created_at: now,
				updated_at: now,
				encrypted_name: bounded_encrypted_name,
				encrypted_attributes: bounded_encrypted_attrs,
				data_hash,
				encryption_key_id: Some(bounded_key_id),
				is_encrypted: true,
				visibility: visibility.clone(),
				authorized_roles: bounded_roles,
			};

			Products::<T>::insert(product_id, product);
			CompanyProducts::<T>::insert(company_id, product_id, ());
			NextProductId::<T>::put(product_id.saturating_add(1));

			// Track category
			Categories::<T>::mutate(&bounded_category, |count| {
				*count = Some(count.map_or(1, |c| c.saturating_add(1)));
			});

			Self::deposit_event(Event::ProductCreatedEncrypted {
				product_id,
				company_id,
				data_hash,
			});

			Ok(())
		}

		/// Get product with access control (logs access attempt)
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn access_product(
			origin: OriginFor<T>,
			product_id: u32,
			user_role: u32,
			user_company_id: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let product = Products::<T>::get(product_id).ok_or(Error::<T>::ProductNotFound)?;

			// Check access control
			let has_access = Self::check_product_access(&product, user_role, user_company_id);
			ensure!(has_access, Error::<T>::NotAuthorized);

			Self::deposit_event(Event::ProductAccessGranted {
				product_id,
				accessor: who,
			});

			Ok(())
		}

		/// Update product visibility
		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn update_visibility(
			origin: OriginFor<T>,
			product_id: u32,
			new_visibility: VisibilityLevel,
			new_authorized_roles: Vec<u32>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let bounded_roles: BoundedVec<u32, T::MaxAuthorizedRoles> =
				new_authorized_roles.try_into().map_err(|_| Error::<T>::TooManyAuthorizedRoles)?;

			Products::<T>::try_mutate(product_id, |maybe_product| -> DispatchResult {
				let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;

				product.visibility = new_visibility.clone();
				product.authorized_roles = bounded_roles;
				product.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::VisibilityUpdated {
				product_id,
				new_visibility,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check if user has access to product based on role and company
		fn check_product_access(
			product: &Product<T>,
			user_role: u32,
			user_company_id: u32,
		) -> bool {
			match product.visibility {
				VisibilityLevel::Public => true,
				VisibilityLevel::Company => user_company_id == product.company_id,
				VisibilityLevel::Management => {
					// Role IDs: 1=Admin, 2=Manager (adjust based on your role-permissions pallet)
					user_company_id == product.company_id && (user_role == 1 || user_role == 2)
				}
				VisibilityLevel::Restricted => {
					user_company_id == product.company_id
						&& product.authorized_roles.contains(&user_role)
				}
				VisibilityLevel::Private => false,
			}
		}
	}
}
