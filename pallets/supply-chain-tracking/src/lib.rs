//! # Supply Chain Tracking Pallet
//!
//! A FRAME pallet for tracking products through the supply chain with append-only, Merkle-like records.
//!
//! ## Overview
//!
//! This pallet provides functionality for:
//! - Creating append-only tracking records for items
//! - Merkle-like chain structure with previous hash links
//! - Immutable audit trail for complete item history
//! - Item ID derivation from chain wallets
//! - Encrypted data storage for privacy
//!
//! ## Append-Only Architecture
//!
//! Each tracking update creates a NEW record linked to the previous one:
//! ```
//! Record 0: { item_id, status, data (encrypted), timestamp, previous_hash: 0x00, hash }
//! Record 1: { item_id, status, data (encrypted), timestamp, previous_hash: hash(Record 0), hash }
//! Record 2: { item_id, status, data (encrypted), timestamp, previous_hash: hash(Record 1), hash }
//! ```
//!
//! Query `get_item_history(item_id)` returns all records in chronological order.

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
	use frame::deps::codec::{Decode, Encode, MaxEncodedLen};
	use frame::deps::sp_io::hashing::blake2_256;
	use scale_info::prelude::vec::Vec;
	use crate::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		/// Maximum length of encrypted data per record
		#[pallet::constant]
		type MaxEncryptedDataLength: Get<u32>;

		/// Maximum length of status string
		#[pallet::constant]
		type MaxStatusLength: Get<u32>;

		/// Maximum length of location string
		#[pallet::constant]
		type MaxLocationLength: Get<u32>;

		/// Maximum number of records to return in history query
		#[pallet::constant]
		type MaxHistoryRecords: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Item tracking record (append-only)
	///
	/// Each update creates a new record linked to the previous one via previous_hash
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ItemRecord<T: Config> {
		/// Unique item ID (derived from chain wallet + item index)
		pub item_id: [u8; 32],
		/// Chain ID this item belongs to
		pub business_id: u32,
		/// Record sequence number (0, 1, 2, ...)
		pub sequence: u32,
		/// Status string (encrypted or plain depending on chain config)
		pub status: BoundedVec<u8, T::MaxStatusLength>,
		/// Current location (optional, encrypted)
		pub location: Option<BoundedVec<u8, T::MaxLocationLength>>,
		/// Encrypted data payload (JSON with custom fields)
		pub encrypted_data: BoundedVec<u8, T::MaxEncryptedDataLength>,
		/// Timestamp of this record
		pub timestamp: BlockNumberFor<T>,
		/// Account that created this record
		pub recorder: T::AccountId,
		/// Hash of previous record (creates Merkle-like chain)
		pub previous_hash: [u8; 32],
		/// Hash of this record
		pub record_hash: [u8; 32],
	}

	/// Item metadata (first record info for quick lookups)
	#[derive(CloneNoBound, Encode, Decode, PartialEqNoBound, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ItemMetadata<T: Config> {
		/// Item ID
		pub item_id: [u8; 32],
		/// Chain ID
		pub business_id: u32,
		/// Total number of records
		pub record_count: u32,
		/// First record timestamp
		pub created_at: BlockNumberFor<T>,
		/// Last record timestamp
		pub last_updated: BlockNumberFor<T>,
		/// Creator account
		pub creator: T::AccountId,
		/// Last hash in the chain
		pub latest_hash: [u8; 32],
	}

	/// Storage: All tracking records (indexed by item_id and sequence)
	/// This creates a double map: item_id → sequence → ItemRecord
	#[pallet::storage]
	#[pallet::getter(fn item_records)]
	pub type ItemRecords<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, [u8; 32],  // item_id
		Blake2_128Concat, u32,        // sequence number
		ItemRecord<T>,
	>;

	/// Storage: Item metadata for quick lookups
	#[pallet::storage]
	#[pallet::getter(fn item_metadata)]
	pub type ItemMetadata_<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], ItemMetadata<T>>;

	/// Storage: Items by chain (for querying all items in a chain)
	#[pallet::storage]
	#[pallet::getter(fn chain_items)]
	pub type BusinessItems<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, u32,       // business_id
		Blake2_128Concat, [u8; 32],  // item_id
		(),
	>;

	/// Storage: Items by location hash (for location-based queries)
	#[pallet::storage]
	#[pallet::getter(fn location_items)]
	pub type LocationItems<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, [u8; 32],  // location_hash
		Blake2_128Concat, [u8; 32],  // item_id
		(),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Item tracking initiated (first record)
		ItemCreated {
			item_id: [u8; 32],
			business_id: u32,
			creator: T::AccountId,
		},
		/// New tracking record appended
		RecordAppended {
			item_id: [u8; 32],
			sequence: u32,
			status: Vec<u8>,
			record_hash: [u8; 32],
			previous_hash: [u8; 32],
		},
		/// Item history queried
		HistoryQueried {
			item_id: [u8; 32],
			record_count: u32,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Item already exists
		ItemAlreadyExists,
		/// Item not found
		ItemNotFound,
		/// Not authorized to update this item
		NotAuthorized,
		/// Encrypted data too long
		EncryptedDataTooLong,
		/// Status string too long
		StatusTooLong,
		/// Location string too long
		LocationTooLong,
		/// Invalid sequence number
		InvalidSequence,
		/// Hash mismatch (chain integrity violated)
		HashMismatch,
		/// Max history records exceeded
		MaxHistoryExceeded,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create the first tracking record for an item
		///
		/// This initiates the append-only chain for an item.
		/// Subsequent updates must use `append_record`.
		///
		/// Parameters:
		/// - item_id: Unique item identifier (derived off-chain)
		/// - business_id: Chain this item belongs to
		/// - status: Initial status (e.g., "Created", "Manufactured")
		/// - location: Optional initial location
		/// - encrypted_data: Encrypted payload with custom fields
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_tracking())]
		pub fn create_item(
			origin: OriginFor<T>,
			item_id: [u8; 32],
			business_id: u32,
			status: Vec<u8>,
			location: Option<Vec<u8>>,
			encrypted_data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure item doesn't already exist
			ensure!(
				!ItemMetadata_::<T>::contains_key(&item_id),
				Error::<T>::ItemAlreadyExists
			);

			// Validate sizes
			let bounded_status: BoundedVec<u8, T::MaxStatusLength> =
				status.clone().try_into().map_err(|_| Error::<T>::StatusTooLong)?;

			let bounded_location = if let Some(loc) = location.clone() {
				Some(loc.try_into().map_err(|_| Error::<T>::LocationTooLong)?)
			} else {
				None
			};

			let bounded_data: BoundedVec<u8, T::MaxEncryptedDataLength> =
				encrypted_data.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			let now = frame_system::Pallet::<T>::block_number();

			// First record has previous_hash = zero
			let previous_hash = [0u8; 32];

			// Create first record
			let record = ItemRecord {
				item_id,
				business_id,
				sequence: 0,
				status: bounded_status,
				location: bounded_location.clone(),
				encrypted_data: bounded_data,
				timestamp: now,
				recorder: who.clone(),
				previous_hash,
				record_hash: [0u8; 32], // Computed below
			};

			// Compute record hash
			let record_hash = Self::compute_record_hash(&record);
			let mut final_record = record;
			final_record.record_hash = record_hash;

			// Store record
			ItemRecords::<T>::insert(&item_id, 0u32, final_record);

			// Create metadata
			let metadata = ItemMetadata {
				item_id,
				business_id,
				record_count: 1,
				created_at: now,
				last_updated: now,
				creator: who.clone(),
				latest_hash: record_hash,
			};

			ItemMetadata_::<T>::insert(&item_id, metadata);

			// Index by chain
			BusinessItems::<T>::insert(business_id, &item_id, ());

			// Index by location if provided
			if let Some(loc) = bounded_location {
				let location_hash = blake2_256(&loc);
				LocationItems::<T>::insert(&location_hash, &item_id, ());
			}

			// Emit event
			Self::deposit_event(Event::ItemCreated {
				item_id,
				business_id,
				creator: who,
			});

			Ok(())
		}

		/// Append a new tracking record to an existing item
		///
		/// This creates an append-only record linked to the previous one.
		/// The previous_hash ensures integrity of the chain.
		///
		/// Parameters:
		/// - item_id: Item to update
		/// - status: New status
		/// - location: New location (optional)
		/// - encrypted_data: New encrypted payload
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_event())]
		pub fn append_record(
			origin: OriginFor<T>,
			item_id: [u8; 32],
			status: Vec<u8>,
			location: Option<Vec<u8>>,
			encrypted_data: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get metadata
			let mut metadata = ItemMetadata_::<T>::get(&item_id)
				.ok_or(Error::<T>::ItemNotFound)?;

			// Validate sizes
			let bounded_status: BoundedVec<u8, T::MaxStatusLength> =
				status.clone().try_into().map_err(|_| Error::<T>::StatusTooLong)?;

			let bounded_location = if let Some(loc) = location.clone() {
				Some(loc.try_into().map_err(|_| Error::<T>::LocationTooLong)?)
			} else {
				None
			};

			let bounded_data: BoundedVec<u8, T::MaxEncryptedDataLength> =
				encrypted_data.try_into().map_err(|_| Error::<T>::EncryptedDataTooLong)?;

			let now = frame_system::Pallet::<T>::block_number();

			// Get previous hash from metadata
			let previous_hash = metadata.latest_hash;
			let sequence = metadata.record_count;

			// Create new record
			let record = ItemRecord {
				item_id,
				business_id: metadata.business_id,
				sequence,
				status: bounded_status,
				location: bounded_location.clone(),
				encrypted_data: bounded_data,
				timestamp: now,
				recorder: who.clone(),
				previous_hash,
				record_hash: [0u8; 32], // Computed below
			};

			// Compute record hash
			let record_hash = Self::compute_record_hash(&record);
			let mut final_record = record;
			final_record.record_hash = record_hash;

			// Store record
			ItemRecords::<T>::insert(&item_id, sequence, final_record);

			// Update metadata
			metadata.record_count = metadata.record_count.saturating_add(1);
			metadata.last_updated = now;
			metadata.latest_hash = record_hash;
			ItemMetadata_::<T>::insert(&item_id, metadata);

			// Update location index if provided
			if let Some(loc) = bounded_location {
				let location_hash = blake2_256(&loc);
				LocationItems::<T>::insert(&location_hash, &item_id, ());
			}

			// Emit event
			Self::deposit_event(Event::RecordAppended {
				item_id,
				sequence,
				status,
				record_hash,
				previous_hash,
			});

			Ok(())
		}

		/// Get complete item history (all records)
		///
		/// This is a read-only query function that returns all records
		/// for an item in chronological order.
		///
		/// Note: This emits an event but doesn't modify state.
		/// Actual data retrieval happens via RPC/runtime API.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::add_event())]
		pub fn query_item_history(
			origin: OriginFor<T>,
			item_id: [u8; 32],
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Get metadata to know record count
			let metadata = ItemMetadata_::<T>::get(&item_id)
				.ok_or(Error::<T>::ItemNotFound)?;

			// Emit event (actual data retrieval via RPC)
			Self::deposit_event(Event::HistoryQueried {
				item_id,
				record_count: metadata.record_count,
			});

			Ok(())
		}

		/// Verify item chain integrity
		///
		/// Checks that all hashes in the chain are correct.
		/// Returns error if any hash mismatch is found.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::add_event())]
		pub fn verify_item_chain(
			origin: OriginFor<T>,
			item_id: [u8; 32],
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Get metadata
			let metadata = ItemMetadata_::<T>::get(&item_id)
				.ok_or(Error::<T>::ItemNotFound)?;

			// Verify each record in sequence
			let mut expected_previous_hash = [0u8; 32];

			for seq in 0..metadata.record_count {
				let record = ItemRecords::<T>::get(&item_id, seq)
					.ok_or(Error::<T>::InvalidSequence)?;

				// Check previous hash matches
				ensure!(
					record.previous_hash == expected_previous_hash,
					Error::<T>::HashMismatch
				);

				// Verify record hash
				let computed_hash = Self::compute_record_hash(&record);
				ensure!(
					record.record_hash == computed_hash,
					Error::<T>::HashMismatch
				);

				// Update expected previous hash for next iteration
				expected_previous_hash = record.record_hash;
			}

			// Final check: last hash should match metadata
			ensure!(
				expected_previous_hash == metadata.latest_hash,
				Error::<T>::HashMismatch
			);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Compute hash of a record (excluding the record_hash field itself)
		///
		/// This creates the Merkle-like link between records
		fn compute_record_hash(record: &ItemRecord<T>) -> [u8; 32] {
			// Encode record without record_hash
			let mut data = Vec::new();
			record.item_id.encode_to(&mut data);
			record.business_id.encode_to(&mut data);
			record.sequence.encode_to(&mut data);
			record.status.encode_to(&mut data);
			record.location.encode_to(&mut data);
			record.encrypted_data.encode_to(&mut data);
			record.timestamp.encode_to(&mut data);
			record.recorder.encode_to(&mut data);
			record.previous_hash.encode_to(&mut data);

			blake2_256(&data)
		}

		/// Get all records for an item (runtime API helper)
		///
		/// This function can be called from runtime APIs to retrieve
		/// the complete history of an item.
		pub fn get_item_history(item_id: [u8; 32]) -> Result<Vec<ItemRecord<T>>, DispatchError> {
			let metadata = ItemMetadata_::<T>::get(&item_id)
				.ok_or(Error::<T>::ItemNotFound)?;

			let mut records = Vec::new();

			for seq in 0..metadata.record_count {
				if let Some(record) = ItemRecords::<T>::get(&item_id, seq) {
					records.push(record);
				}
			}

			Ok(records)
		}

		/// Get items by chain ID (helper function)
		pub fn get_chain_items(business_id: u32) -> Vec<[u8; 32]> {
			BusinessItems::<T>::iter_prefix(business_id)
				.map(|(item_id, _)| item_id)
				.collect()
		}

		/// Get items at location (helper function)
		pub fn get_items_at_location(location_hash: [u8; 32]) -> Vec<[u8; 32]> {
			LocationItems::<T>::iter_prefix(location_hash)
				.map(|(item_id, _)| item_id)
				.collect()
		}
	}
}
