//! # Supply Chain Tracking Pallet
//!
//! A FRAME pallet for tracking products through the supply chain.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating tracking records for products
//! - Adding events to track product journey (Manufactured, Shipped, InTransit, Delivered, QualityCheck, Delayed)
//! - Updating product status and location
//! - Querying tracking history
//! - Location-based product tracking

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
	use frame::prelude::*;
	use frame::deps::codec::{Decode, Encode, MaxEncodedLen, DecodeWithMemTracking};
	use scale_info::prelude::vec::Vec;
	use crate::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxLocationLength: Get<u32>;

		#[pallet::constant]
		type MaxNotesLength: Get<u32>;

		#[pallet::constant]
		type MaxEvents: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Event type for tracking
	#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum EventType {
		Manufactured,
		Shipped,
		InTransit,
		Delivered,
		QualityCheck,
		Delayed,
	}

	/// Tracking event
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct TrackingEvent<T: Config> {
		pub event_type: EventType,
		pub location: BoundedVec<u8, T::MaxLocationLength>,
		pub timestamp: BlockNumberFor<T>,
		pub recorder: T::AccountId,
		pub notes: BoundedVec<u8, T::MaxNotesLength>,
	}

	/// Tracking status
	#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum TrackingStatus {
		Created,
		InProgress,
		Completed,
		OnHold,
		Cancelled,
	}

	/// Tracking record for a product
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct TrackingRecord<T: Config> {
		pub product_id: u32,
		pub current_status: TrackingStatus,
		pub current_location: BoundedVec<u8, T::MaxLocationLength>,
		pub events: BoundedVec<TrackingEvent<T>, T::MaxEvents>,
		pub created_at: BlockNumberFor<T>,
		pub updated_at: BlockNumberFor<T>,
	}

	/// Storage: Tracking records indexed by product ID
	#[pallet::storage]
	#[pallet::getter(fn tracking_records)]
	pub type TrackingRecords<T: Config> = StorageMap<_, Blake2_128Concat, u32, TrackingRecord<T>>;

	/// Storage: Product tracking by company
	#[pallet::storage]
	#[pallet::getter(fn product_tracking)]
	pub type ProductTracking<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, u32,  // company_id
		Blake2_128Concat, u32,  // product_id
		(),
	>;

	/// Storage: Products at specific locations
	#[pallet::storage]
	#[pallet::getter(fn location_products)]
	pub type LocationProducts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, [u8; 32],  // location_hash
		Blake2_128Concat, u32,       // product_id
		(),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TrackingCreated { product_id: u32, location: Vec<u8> },
		EventAdded { product_id: u32, event_type: EventType },
		StatusUpdated { product_id: u32, status: TrackingStatus },
		LocationUpdated { product_id: u32, new_location: Vec<u8> },
	}

	#[pallet::error]
	pub enum Error<T> {
		RecordNotFound,
		NotAuthorized,
		LocationTooLong,
		NotesTooLong,
		MaxEventsReached,
		TrackingAlreadyExists,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new tracking record
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_tracking())]
		pub fn create_tracking(
			origin: OriginFor<T>,
			product_id: u32,
			company_id: u32,
			initial_location: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!TrackingRecords::<T>::contains_key(product_id), Error::<T>::TrackingAlreadyExists);

			let bounded_location: BoundedVec<u8, T::MaxLocationLength> =
				initial_location.clone().try_into().map_err(|_| Error::<T>::LocationTooLong)?;

			let now = frame_system::Pallet::<T>::block_number();

			let tracking_record = TrackingRecord {
				product_id,
				current_status: TrackingStatus::Created,
				current_location: bounded_location.clone(),
				events: BoundedVec::default(),
				created_at: now,
				updated_at: now,
			};

			TrackingRecords::<T>::insert(product_id, tracking_record);
			ProductTracking::<T>::insert(company_id, product_id, ());

			// Add to location index
			let location_hash = Self::hash_location(&initial_location);
			LocationProducts::<T>::insert(location_hash, product_id, ());

			Self::deposit_event(Event::TrackingCreated { product_id, location: initial_location });

			Ok(())
		}

		/// Add a tracking event
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_event())]
		pub fn add_event(
			origin: OriginFor<T>,
			product_id: u32,
			event_type: EventType,
			location: Vec<u8>,
			notes: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_location: BoundedVec<u8, T::MaxLocationLength> =
				location.clone().try_into().map_err(|_| Error::<T>::LocationTooLong)?;

			let bounded_notes: BoundedVec<u8, T::MaxNotesLength> =
				notes.try_into().map_err(|_| Error::<T>::NotesTooLong)?;

			TrackingRecords::<T>::try_mutate(product_id, |maybe_record| -> DispatchResult {
				let record = maybe_record.as_mut().ok_or(Error::<T>::RecordNotFound)?;

				let now = frame_system::Pallet::<T>::block_number();

				let event = TrackingEvent {
					event_type: event_type.clone(),
					location: bounded_location.clone(),
					timestamp: now,
					recorder: who.clone(),
					notes: bounded_notes,
				};

				record.events.try_push(event).map_err(|_| Error::<T>::MaxEventsReached)?;
				record.updated_at = now;

				// Update location if different
				if record.current_location != bounded_location {
					// Remove from old location index
					let old_location_hash = Self::hash_location(&record.current_location.to_vec());
					LocationProducts::<T>::remove(old_location_hash, product_id);

					// Add to new location index
					let new_location_hash = Self::hash_location(&location);
					LocationProducts::<T>::insert(new_location_hash, product_id, ());

					record.current_location = bounded_location;
				}

				Ok(())
			})?;

			Self::deposit_event(Event::EventAdded { product_id, event_type });

			Ok(())
		}

		/// Update tracking status
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::update_status())]
		pub fn update_status(
			origin: OriginFor<T>,
			product_id: u32,
			new_status: TrackingStatus,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			TrackingRecords::<T>::try_mutate(product_id, |maybe_record| -> DispatchResult {
				let record = maybe_record.as_mut().ok_or(Error::<T>::RecordNotFound)?;
				record.current_status = new_status.clone();
				record.updated_at = frame_system::Pallet::<T>::block_number();
				Ok(())
			})?;

			Self::deposit_event(Event::StatusUpdated { product_id, status: new_status });

			Ok(())
		}

		/// Update current location
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_location())]
		pub fn update_location(
			origin: OriginFor<T>,
			product_id: u32,
			new_location: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let bounded_location: BoundedVec<u8, T::MaxLocationLength> =
				new_location.clone().try_into().map_err(|_| Error::<T>::LocationTooLong)?;

			TrackingRecords::<T>::try_mutate(product_id, |maybe_record| -> DispatchResult {
				let record = maybe_record.as_mut().ok_or(Error::<T>::RecordNotFound)?;

				// Remove from old location index
				let old_location_hash = Self::hash_location(&record.current_location.to_vec());
				LocationProducts::<T>::remove(old_location_hash, product_id);

				// Update location
				record.current_location = bounded_location;
				record.updated_at = frame_system::Pallet::<T>::block_number();

				// Add to new location index
				let new_location_hash = Self::hash_location(&new_location);
				LocationProducts::<T>::insert(new_location_hash, product_id, ());

				Ok(())
			})?;

			Self::deposit_event(Event::LocationUpdated { product_id, new_location });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Hash a location string for indexing
		fn hash_location(location: &[u8]) -> [u8; 32] {
			use frame::traits::Hash;
			let hash = <T::Hashing as Hash>::hash(location);
			let mut result = [0u8; 32];
			result.copy_from_slice(hash.as_ref());
			result
		}
	}
}
