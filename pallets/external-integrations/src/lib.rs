#![cfg_attr(not(feature = "std"), no_std)]

//! # External Integrations Pallet
//!
//! This pallet provides external integration capabilities for the supply chain system:
//! - REST API access with authentication and rate limiting
//! - Bulk import/export functionality for data migration
//! - Email notification system for alerts and updates
//! - Barcode generation and scanning support
//!
//! ## Overview
//!
//! The External Integrations pallet enables third-party systems to interact with the
//! blockchain through secure APIs, allows for batch data operations, sends notifications
//! to users, and supports barcode-based product tracking.

use scale_info::prelude::vec::Vec;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame::prelude::*;
	use scale_info::prelude::{vec, vec::Vec};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configuration trait for the external integrations pallet
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics
		type WeightInfo: WeightInfo;

		/// Maximum length for API keys
		#[pallet::constant]
		type MaxApiKeyLength: Get<u32>;

		/// Maximum number of API keys per account
		#[pallet::constant]
		type MaxApiKeysPerAccount: Get<u32>;

		/// Maximum batch size for import/export operations
		#[pallet::constant]
		type MaxBatchSize: Get<u32>;

		/// Maximum email recipients per notification
		#[pallet::constant]
		type MaxEmailRecipients: Get<u32>;

		/// Maximum barcode data length
		#[pallet::constant]
		type MaxBarcodeLength: Get<u32>;
	}

	// ===== Storage Items =====

	/// API Keys: Maps API key hash to API key details
	#[pallet::storage]
	#[pallet::getter(fn api_keys)]
	pub type ApiKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // API key hash
		ApiKeyInfo<T>,
		OptionQuery,
	>;

	/// User API Keys: Maps account to their API keys
	#[pallet::storage]
	#[pallet::getter(fn user_api_keys)]
	pub type UserApiKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxApiKeysPerAccount>,
		ValueQuery,
	>;

	/// API Rate Limits: Tracks API usage per key per block
	#[pallet::storage]
	#[pallet::getter(fn api_rate_limits)]
	pub type ApiRateLimits<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // API key hash
		RateLimitInfo<BlockNumberFor<T>>,
		ValueQuery,
	>;

	/// Bulk Import Jobs: Tracks import job status
	#[pallet::storage]
	#[pallet::getter(fn import_jobs)]
	pub type ImportJobs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Job ID
		ImportJobInfo<T>,
		OptionQuery,
	>;

	/// Export Jobs: Tracks export job status
	#[pallet::storage]
	#[pallet::getter(fn export_jobs)]
	pub type ExportJobs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Job ID
		ExportJobInfo<T>,
		OptionQuery,
	>;

	/// Email Templates: Stores email templates for notifications
	#[pallet::storage]
	#[pallet::getter(fn email_templates)]
	pub type EmailTemplates<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		EmailTemplateType,
		EmailTemplate<T>,
		OptionQuery,
	>;

	/// Email Queue: Pending emails to be sent
	#[pallet::storage]
	#[pallet::getter(fn email_queue)]
	pub type EmailQueue<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Email ID
		EmailNotification<T>,
		OptionQuery,
	>;

	/// Email Delivery Status: Tracks email delivery
	#[pallet::storage]
	#[pallet::getter(fn email_delivery_status)]
	pub type EmailDeliveryStatus<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Email ID
		DeliveryStatus<BlockNumberFor<T>>,
		OptionQuery,
	>;

	/// Barcode Registry: Maps barcode to product/entity ID
	#[pallet::storage]
	#[pallet::getter(fn barcode_registry)]
	pub type BarcodeRegistry<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxBarcodeLength>,
		BarcodeInfo<T>,
		OptionQuery,
	>;

	/// Webhook Endpoints: Stores webhook URLs for external notifications
	#[pallet::storage]
	#[pallet::getter(fn webhooks)]
	pub type Webhooks<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<WebhookEndpoint<T>, T::MaxApiKeysPerAccount>,
		ValueQuery,
	>;

	// ===== Events =====

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// API key created [account, key_hash, name]
		ApiKeyCreated { account: T::AccountId, key_hash: T::Hash, name: Vec<u8> },

		/// API key revoked [account, key_hash]
		ApiKeyRevoked { account: T::AccountId, key_hash: T::Hash },

		/// Rate limit exceeded [key_hash, current_count, limit]
		RateLimitExceeded { key_hash: T::Hash, current_count: u32, limit: u32 },

		/// Bulk import job created [job_id, account, item_count]
		ImportJobCreated { job_id: T::Hash, account: T::AccountId, item_count: u32 },

		/// Import job completed [job_id, success_count, failure_count]
		ImportJobCompleted { job_id: T::Hash, success_count: u32, failure_count: u32 },

		/// Export job created [job_id, account, export_type]
		ExportJobCreated { job_id: T::Hash, account: T::AccountId, export_type: ExportType },

		/// Export job completed [job_id, record_count]
		ExportJobCompleted { job_id: T::Hash, record_count: u32 },

		/// Email notification queued [email_id, recipient, template_type]
		EmailQueued { email_id: T::Hash, recipient: Vec<u8>, template_type: EmailTemplateType },

		/// Email sent successfully [email_id]
		EmailSent { email_id: T::Hash },

		/// Email delivery failed [email_id, reason]
		EmailFailed { email_id: T::Hash, reason: Vec<u8> },

		/// Email template created/updated [template_type, name]
		EmailTemplateUpdated { template_type: EmailTemplateType, name: Vec<u8> },

		/// Barcode registered [barcode, entity_type, entity_id]
		BarcodeRegistered { barcode: Vec<u8>, entity_type: BarcodeEntityType, entity_id: T::Hash },

		/// Barcode scanned [barcode, scanner, timestamp]
		BarcodeScanned { barcode: Vec<u8>, scanner: T::AccountId, timestamp: u64 },

		/// Webhook registered [account, url, event_types]
		WebhookRegistered { account: T::AccountId, url: Vec<u8>, event_types: Vec<WebhookEventType> },

		/// Webhook triggered [account, url, event_type]
		WebhookTriggered { account: T::AccountId, url: Vec<u8>, event_type: WebhookEventType },
	}

	// ===== Errors =====

	#[pallet::error]
	pub enum Error<T> {
		/// API key already exists
		ApiKeyAlreadyExists,

		/// API key not found
		ApiKeyNotFound,

		/// Maximum API keys reached
		MaxApiKeysReached,

		/// Rate limit exceeded
		RateLimitExceeded,

		/// Invalid API key
		InvalidApiKey,

		/// Unauthorized access
		Unauthorized,

		/// Import job not found
		ImportJobNotFound,

		/// Export job not found
		ExportJobNotFound,

		/// Batch size exceeded
		BatchSizeExceeded,

		/// Invalid batch data
		InvalidBatchData,

		/// Email template not found
		EmailTemplateNotFound,

		/// Invalid email address
		InvalidEmailAddress,

		/// Email queue full
		EmailQueueFull,

		/// Too many recipients
		TooManyRecipients,

		/// Email sending failed
		EmailSendingFailed,

		/// Barcode already registered
		BarcodeAlreadyRegistered,

		/// Barcode not found
		BarcodeNotFound,

		/// Invalid barcode format
		InvalidBarcodeFormat,

		/// Webhook already exists
		WebhookAlreadyExists,

		/// Webhook not found
		WebhookNotFound,

		/// Invalid webhook URL
		InvalidWebhookUrl,
	}

	// ===== Extrinsics =====

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new API key for external access
		///
		/// Parameters:
		/// - `origin`: The account creating the API key
		/// - `name`: Human-readable name for the API key
		/// - `permissions`: List of permissions for this key
		/// - `rate_limit`: Maximum requests per block
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_api_key())]
		pub fn create_api_key(
			origin: OriginFor<T>,
			name: Vec<u8>,
			permissions: Vec<ApiPermission>,
			rate_limit: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if user has reached max API keys
			let user_keys = UserApiKeys::<T>::get(&who);
			ensure!(
				user_keys.len() < T::MaxApiKeysPerAccount::get() as usize,
				Error::<T>::MaxApiKeysReached
			);

			// Generate unique API key hash
			let key_data = (who.clone(), name.clone(), frame_system::Pallet::<T>::block_number());
			let key_hash = T::Hashing::hash_of(&key_data);

			// Ensure key doesn't already exist
			ensure!(!ApiKeys::<T>::contains_key(&key_hash), Error::<T>::ApiKeyAlreadyExists);

			// Create API key info
			let api_key_info = ApiKeyInfo {
				owner: who.clone(),
				name: name.clone().try_into().map_err(|_| Error::<T>::InvalidApiKey)?,
				permissions: permissions.try_into().map_err(|_| Error::<T>::InvalidApiKey)?,
				rate_limit,
				created_at: frame_system::Pallet::<T>::block_number(),
				last_used: None,
				is_active: true,
			};

			// Store API key
			ApiKeys::<T>::insert(&key_hash, api_key_info);

			// Add to user's API keys
			UserApiKeys::<T>::try_mutate(&who, |keys| {
				keys.try_push(key_hash)
					.map_err(|_| Error::<T>::MaxApiKeysReached)
			})?;

			// Emit event
			Self::deposit_event(Event::ApiKeyCreated {
				account: who,
				key_hash,
				name,
			});

			Ok(())
		}

		/// Revoke an existing API key
		///
		/// Parameters:
		/// - `origin`: The account revoking the API key (must be owner)
		/// - `key_hash`: Hash of the API key to revoke
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::revoke_api_key())]
		pub fn revoke_api_key(
			origin: OriginFor<T>,
			key_hash: T::Hash,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get API key info
			let api_key = ApiKeys::<T>::get(&key_hash)
				.ok_or(Error::<T>::ApiKeyNotFound)?;

			// Ensure caller is the owner
			ensure!(api_key.owner == who, Error::<T>::Unauthorized);

			// Remove from storage
			ApiKeys::<T>::remove(&key_hash);
			ApiRateLimits::<T>::remove(&key_hash);

			// Remove from user's key list
			UserApiKeys::<T>::mutate(&who, |keys| {
				keys.retain(|k| k != &key_hash);
			});

			// Emit event
			Self::deposit_event(Event::ApiKeyRevoked {
				account: who,
				key_hash,
			});

			Ok(())
		}

		/// Create a bulk import job
		///
		/// Parameters:
		/// - `origin`: The account creating the import job
		/// - `import_type`: Type of data being imported
		/// - `data_hash`: Hash of the import data (stored off-chain)
		/// - `item_count`: Number of items in the batch
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_import_job())]
		pub fn create_import_job(
			origin: OriginFor<T>,
			import_type: ImportType,
			data_hash: T::Hash,
			item_count: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate batch size
			ensure!(
				item_count <= T::MaxBatchSize::get(),
				Error::<T>::BatchSizeExceeded
			);

			// Generate job ID
			let job_id = T::Hashing::hash_of(&(who.clone(), data_hash, frame_system::Pallet::<T>::block_number()));

			// Create import job
			let import_job = ImportJobInfo {
				creator: who.clone(),
				import_type,
				data_hash,
				item_count,
				processed_count: 0,
				success_count: 0,
				failure_count: 0,
				status: JobStatus::Pending,
				created_at: frame_system::Pallet::<T>::block_number(),
				completed_at: None,
			};

			// Store job
			ImportJobs::<T>::insert(&job_id, import_job);

			// Emit event
			Self::deposit_event(Event::ImportJobCreated {
				job_id,
				account: who,
				item_count,
			});

			Ok(())
		}

		/// Create a bulk export job
		///
		/// Parameters:
		/// - `origin`: The account creating the export job
		/// - `export_type`: Type of data to export
		/// - `filters`: Optional filters for export
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::create_export_job())]
		pub fn create_export_job(
			origin: OriginFor<T>,
			export_type: ExportType,
			format: ExportFormat,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Generate job ID
			let job_id = T::Hashing::hash_of(&(who.clone(), export_type.clone(), frame_system::Pallet::<T>::block_number()));

			// Create export job
			let export_job = ExportJobInfo {
				creator: who.clone(),
				export_type: export_type.clone(),
				format,
				record_count: 0,
				status: JobStatus::Pending,
				created_at: frame_system::Pallet::<T>::block_number(),
				completed_at: None,
				download_url: None,
			};

			// Store job
			ExportJobs::<T>::insert(&job_id, export_job);

			// Emit event
			Self::deposit_event(Event::ExportJobCreated {
				job_id,
				account: who,
				export_type,
			});

			Ok(())
		}

		/// Queue an email notification
		///
		/// Parameters:
		/// - `origin`: The account queuing the email
		/// - `recipients`: List of email addresses
		/// - `template_type`: Type of email template to use
		/// - `variables`: Template variables
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::queue_email())]
		pub fn queue_email(
			origin: OriginFor<T>,
			recipients: Vec<Vec<u8>>,
			template_type: EmailTemplateType,
			variables: Vec<(Vec<u8>, Vec<u8>)>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate recipients count
			ensure!(
				recipients.len() <= T::MaxEmailRecipients::get() as usize,
				Error::<T>::EmailQueueFull
			);

			// Ensure template exists
			ensure!(
				EmailTemplates::<T>::contains_key(&template_type),
				Error::<T>::EmailTemplateNotFound
			);

			// Create email notification
			// Convert Vec<Vec<u8>> to BoundedVec<BoundedVec<u8, ...>, ...>
			let bounded_recipients: BoundedVec<BoundedVec<u8, ConstU32<128>>, T::MaxEmailRecipients> = recipients
				.into_iter()
				.map(|r| r.try_into().map_err(|_| Error::<T>::InvalidEmailAddress))
				.collect::<Result<Vec<_>, _>>()?
				.try_into()
				.map_err(|_| Error::<T>::TooManyRecipients)?;

			// Generate email ID
			let email_id = T::Hashing::hash_of(&(who.clone(), &bounded_recipients, frame_system::Pallet::<T>::block_number()));

			let bounded_variables: BoundedVec<(BoundedVec<u8, ConstU32<64>>, BoundedVec<u8, ConstU32<256>>), ConstU32<32>> = variables
				.into_iter()
				.map(|(k, v)| -> Result<(BoundedVec<u8, ConstU32<64>>, BoundedVec<u8, ConstU32<256>>), Error<T>> {
					let bk: BoundedVec<u8, ConstU32<64>> = k.try_into().map_err(|_| Error::<T>::InvalidEmailAddress)?;
					let bv: BoundedVec<u8, ConstU32<256>> = v.try_into().map_err(|_| Error::<T>::InvalidEmailAddress)?;
					Ok((bk, bv))
				})
				.collect::<Result<Vec<_>, _>>()?
				.try_into()
				.map_err(|_| Error::<T>::InvalidEmailAddress)?;

			// Save first recipient for event before moving
			let first_recipient = bounded_recipients.get(0).cloned().unwrap_or_default().to_vec();

			let email = EmailNotification {
				sender: who.clone(),
				recipients: bounded_recipients,
				template_type: template_type.clone(),
				variables: bounded_variables,
				queued_at: frame_system::Pallet::<T>::block_number(),
				retry_count: 0,
			};

			// Store email
			EmailQueue::<T>::insert(&email_id, email);

			// Initialize delivery status
			let delivery_status = DeliveryStatus {
				status: EmailStatus::Queued,
				attempts: 0,
				last_attempt: None,
				error_message: None,
			};
			EmailDeliveryStatus::<T>::insert(&email_id, delivery_status);

			// Emit event
			Self::deposit_event(Event::EmailQueued {
				email_id,
				recipient: first_recipient,
				template_type,
			});

			Ok(())
		}

		/// Create or update an email template
		///
		/// Parameters:
		/// - `origin`: The account creating the template (requires admin)
		/// - `template_type`: Type of template
		/// - `name`: Template name
		/// - `subject`: Email subject
		/// - `body`: Email body with variable placeholders
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::update_email_template())]
		pub fn update_email_template(
			origin: OriginFor<T>,
			template_type: EmailTemplateType,
			name: Vec<u8>,
			subject: Vec<u8>,
			body: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// TODO: Add admin permission check

			// Create template
			let template = EmailTemplate {
				name: name.clone(),
				subject,
				body,
				created_at: frame_system::Pallet::<T>::block_number(),
			};

			// Store template
			EmailTemplates::<T>::insert(&template_type, template);

			// Emit event
			Self::deposit_event(Event::EmailTemplateUpdated {
				template_type,
				name,
			});

			Ok(())
		}

		/// Register a barcode for a product or entity
		///
		/// Parameters:
		/// - `origin`: The account registering the barcode
		/// - `barcode`: Barcode data
		/// - `entity_type`: Type of entity (product, shipment, etc.)
		/// - `entity_id`: ID of the entity
		/// - `barcode_type`: Type of barcode (QR, Code128, EAN, etc.)
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::register_barcode())]
		pub fn register_barcode(
			origin: OriginFor<T>,
			barcode: Vec<u8>,
			entity_type: BarcodeEntityType,
			entity_id: T::Hash,
			barcode_type: BarcodeType,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate barcode length
			let bounded_barcode: BoundedVec<u8, T::MaxBarcodeLength> = barcode.clone()
				.try_into()
				.map_err(|_| Error::<T>::InvalidBarcodeFormat)?;

			// Ensure barcode doesn't already exist
			ensure!(
				!BarcodeRegistry::<T>::contains_key(&bounded_barcode),
				Error::<T>::BarcodeAlreadyRegistered
			);

			// Create barcode info
			let barcode_info = BarcodeInfo {
				entity_type: entity_type.clone(),
				entity_id,
				barcode_type,
				created_by: who.clone(),
				created_at: frame_system::Pallet::<T>::block_number(),
				scan_count: 0,
				last_scanned: None,
			};

			// Store barcode
			BarcodeRegistry::<T>::insert(&bounded_barcode, barcode_info);

			// Emit event
			Self::deposit_event(Event::BarcodeRegistered {
				barcode,
				entity_type,
				entity_id,
			});

			Ok(())
		}

		/// Record a barcode scan
		///
		/// Parameters:
		/// - `origin`: The account scanning the barcode
		/// - `barcode`: Barcode data
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::scan_barcode())]
		pub fn scan_barcode(
			origin: OriginFor<T>,
			barcode: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate barcode length
			let bounded_barcode: BoundedVec<u8, T::MaxBarcodeLength> = barcode.clone()
				.try_into()
				.map_err(|_| Error::<T>::InvalidBarcodeFormat)?;

			// Get barcode info
			BarcodeRegistry::<T>::try_mutate(&bounded_barcode, |barcode_info| -> DispatchResult {
				let info = barcode_info.as_mut().ok_or(Error::<T>::BarcodeNotFound)?;

				// Update scan statistics
				info.scan_count = info.scan_count.saturating_add(1);
				info.last_scanned = Some((who.clone(), frame_system::Pallet::<T>::block_number()));

				Ok(())
			})?;

			// Get current timestamp (block number as proxy)
			let timestamp = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();

			// Emit event
			Self::deposit_event(Event::BarcodeScanned {
				barcode,
				scanner: who,
				timestamp,
			});

			Ok(())
		}

		/// Register a webhook endpoint
		///
		/// Parameters:
		/// - `origin`: The account registering the webhook
		/// - `url`: Webhook URL
		/// - `event_types`: List of events to trigger webhook
		/// - `secret`: Optional secret for webhook verification
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::register_webhook())]
		pub fn register_webhook(
			origin: OriginFor<T>,
			url: Vec<u8>,
			event_types: Vec<WebhookEventType>,
			secret: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Create webhook endpoint
			let webhook = WebhookEndpoint {
				url: url.clone(),
				event_types: event_types.clone().try_into().map_err(|_| Error::<T>::InvalidWebhookUrl)?,
				secret,
				is_active: true,
				created_at: frame_system::Pallet::<T>::block_number(),
			};

			// Add to user's webhooks
			Webhooks::<T>::try_mutate(&who, |webhooks| {
				webhooks.try_push(webhook)
					.map_err(|_| Error::<T>::WebhookAlreadyExists)
			})?;

			// Emit event
			Self::deposit_event(Event::WebhookRegistered {
				account: who,
				url,
				event_types,
			});

			Ok(())
		}
	}

	// ===== Helper Functions =====

	impl<T: Config> Pallet<T> {
		/// Verify API key and check rate limit
		pub fn verify_api_key(key_hash: &T::Hash) -> Result<(), Error<T>> {
			// Get API key info
			let api_key = ApiKeys::<T>::get(key_hash)
				.ok_or(Error::<T>::ApiKeyNotFound)?;

			// Check if active
			ensure!(api_key.is_active, Error::<T>::InvalidApiKey);

			// Check rate limit
			let current_block = frame_system::Pallet::<T>::block_number();
			let rate_limit = ApiRateLimits::<T>::get(key_hash);

			if rate_limit.last_reset == current_block {
				ensure!(
					rate_limit.request_count < api_key.rate_limit,
					Error::<T>::RateLimitExceeded
				);
			}

			// Update rate limit
			ApiRateLimits::<T>::mutate(key_hash, |limit| {
				if limit.last_reset == current_block {
					limit.request_count = limit.request_count.saturating_add(1);
				} else {
					limit.last_reset = current_block;
					limit.request_count = 1;
				}
			});

			// Update last used
			ApiKeys::<T>::mutate(key_hash, |key| {
				if let Some(k) = key {
					k.last_used = Some(current_block);
				}
			});

			Ok(())
		}

		/// Check if account has API permission
		pub fn has_api_permission(
			key_hash: &T::Hash,
			permission: ApiPermission,
		) -> bool {
			if let Some(api_key) = ApiKeys::<T>::get(key_hash) {
				api_key.permissions.contains(&permission)
			} else {
				false
			}
		}

		/// Trigger webhooks for an event
		pub fn trigger_webhooks(
			account: &T::AccountId,
			event_type: WebhookEventType,
			payload: Vec<u8>,
		) {
			let webhooks = Webhooks::<T>::get(account);
			for webhook in webhooks.iter() {
				if webhook.is_active && webhook.event_types.contains(&event_type) {
					// In a real implementation, this would make an off-chain HTTP request
					// For now, we just emit an event
					Self::deposit_event(Event::WebhookTriggered {
						account: account.clone(),
						url: webhook.url.clone(),
						event_type: event_type.clone(),
					});
				}
			}
		}
	}
}

// ===== Type Definitions =====

use codec::{Decode, Encode};
use frame::prelude::*;
use scale_info::TypeInfo;

/// API Key information
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ApiKeyInfo<T: Config> {
	pub owner: T::AccountId,
	pub name: BoundedVec<u8, ConstU32<64>>,
	pub permissions: BoundedVec<ApiPermission, ConstU32<32>>,
	pub rate_limit: u32,
	pub created_at: BlockNumberFor<T>,
	pub last_used: Option<BlockNumberFor<T>>,
	pub is_active: bool,
}

/// API permissions
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ApiPermission {
	ReadProducts,
	WriteProducts,
	ReadTracking,
	WriteTracking,
	ReadUsers,
	WriteUsers,
	ReadCompanies,
	WriteCompanies,
	ManageApiKeys,
	BulkOperations,
	WebhookAccess,
}

/// Rate limit tracking
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RateLimitInfo<BlockNumber> {
	pub last_reset: BlockNumber,
	pub request_count: u32,
}

impl<BlockNumber: Default> Default for RateLimitInfo<BlockNumber> {
	fn default() -> Self {
		Self {
			last_reset: Default::default(),
			request_count: 0,
		}
	}
}

/// Import job information
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ImportJobInfo<T: Config> {
	pub creator: T::AccountId,
	pub import_type: ImportType,
	pub data_hash: T::Hash,
	pub item_count: u32,
	pub processed_count: u32,
	pub success_count: u32,
	pub failure_count: u32,
	pub status: JobStatus,
	pub created_at: BlockNumberFor<T>,
	pub completed_at: Option<BlockNumberFor<T>>,
}

/// Import data type
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ImportType {
	Products,
	Users,
	Companies,
	TrackingEntries,
	Barcodes,
}

/// Export job information
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ExportJobInfo<T: Config> {
	pub creator: T::AccountId,
	pub export_type: ExportType,
	pub format: ExportFormat,
	pub record_count: u32,
	pub status: JobStatus,
	pub created_at: BlockNumberFor<T>,
	pub completed_at: Option<BlockNumberFor<T>>,
	pub download_url: Option<BoundedVec<u8, ConstU32<256>>>,
}

/// Export data type
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ExportType {
	Products,
	Users,
	Companies,
	TrackingEntries,
	AuditLogs,
	FullBackup,
}

/// Export format
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ExportFormat {
	CSV,
	JSON,
	Excel,
	PDF,
}

/// Job status
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum JobStatus {
	Pending,
	Processing,
	Completed,
	Failed,
	Cancelled,
}

/// Email notification
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct EmailNotification<T: Config> {
	pub sender: T::AccountId,
	pub recipients: BoundedVec<BoundedVec<u8, ConstU32<128>>, T::MaxEmailRecipients>,
	pub template_type: EmailTemplateType,
	pub variables: BoundedVec<(BoundedVec<u8, ConstU32<64>>, BoundedVec<u8, ConstU32<256>>), ConstU32<32>>,
	pub queued_at: BlockNumberFor<T>,
	pub retry_count: u8,
}

/// Email template
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct EmailTemplate<T: Config> {
	pub name: Vec<u8>,
	pub subject: Vec<u8>,
	pub body: Vec<u8>,
	pub created_at: BlockNumberFor<T>,
}

/// Email template types
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EmailTemplateType {
	UserInvitation,
	ProductUpdate,
	ShipmentAlert,
	DeliveryConfirmation,
	QualityAlert,
	SystemNotification,
	Custom,
}

/// Email delivery status
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct DeliveryStatus<BlockNumber> {
	pub status: EmailStatus,
	pub attempts: u8,
	pub last_attempt: Option<BlockNumber>,
	pub error_message: Option<BoundedVec<u8, ConstU32<256>>>,
}

/// Email status
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EmailStatus {
	Queued,
	Sending,
	Sent,
	Failed,
	Bounced,
}

/// Barcode information
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct BarcodeInfo<T: Config> {
	pub entity_type: BarcodeEntityType,
	pub entity_id: T::Hash,
	pub barcode_type: BarcodeType,
	pub created_by: T::AccountId,
	pub created_at: BlockNumberFor<T>,
	pub scan_count: u32,
	pub last_scanned: Option<(T::AccountId, BlockNumberFor<T>)>,
}

/// Barcode entity type
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum BarcodeEntityType {
	Product,
	Shipment,
	Location,
	Asset,
	Custom,
}

/// Barcode type
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum BarcodeType {
	QRCode,
	Code128,
	Code39,
	EAN13,
	EAN8,
	UPC,
	DataMatrix,
	PDF417,
}

/// Webhook endpoint
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct WebhookEndpoint<T: Config> {
	pub url: Vec<u8>,
	pub event_types: BoundedVec<WebhookEventType, ConstU32<16>>,
	pub secret: Option<Vec<u8>>,
	pub is_active: bool,
	pub created_at: BlockNumberFor<T>,
}

/// Webhook event types
#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum WebhookEventType {
	ProductCreated,
	ProductUpdated,
	TrackingUpdated,
	ShipmentDelivered,
	QualityAlert,
	UserInvited,
	CompanyUpdated,
	All,
}
