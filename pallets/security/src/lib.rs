#![cfg_attr(not(feature = "std"), no_std)]

//! # Security Pallet
//!
//! This pallet provides comprehensive security features for the supply chain system:
//! - Data encryption and key management
//! - Comprehensive audit logging
//! - Automated backup and recovery
//! - Advanced authentication (MFA, OAuth, session management)
//!
//! ## Overview
//!
//! The Security pallet ensures data protection, tracks all system activities,
//! provides disaster recovery capabilities, and implements enterprise-grade
//! authentication mechanisms.

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

	/// Configuration trait for the security pallet
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics
		type WeightInfo: WeightInfo;

		/// Maximum length for encryption keys
		#[pallet::constant]
		type MaxEncryptionKeyLength: Get<u32>;

		/// Maximum audit log entries per account
		#[pallet::constant]
		type MaxAuditLogsPerAccount: Get<u32>;

		/// Maximum backup snapshots to retain
		#[pallet::constant]
		type MaxBackupSnapshots: Get<u32>;

		/// Maximum MFA devices per account
		#[pallet::constant]
		type MaxMfaDevices: Get<u32>;

		/// Maximum OAuth providers
		#[pallet::constant]
		type MaxOAuthProviders: Get<u32>;

		/// Session timeout in blocks
		#[pallet::constant]
		type SessionTimeout: Get<BlockNumberFor<Self>>;
	}

	// ===== Storage Items =====

	// --- Data Encryption ---

	/// Encryption Keys: Maps account to their encryption keys
	#[pallet::storage]
	#[pallet::getter(fn encryption_keys)]
	pub type EncryptionKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		EncryptionKeyInfo<T>,
		OptionQuery,
	>;

	/// Data Encryption Status: Tracks which data is encrypted
	#[pallet::storage]
	#[pallet::getter(fn encryption_status)]
	pub type EncryptionStatus<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Data hash
		DataEncryptionInfo<T>,
		OptionQuery,
	>;

	/// System Encryption Config
	#[pallet::storage]
	#[pallet::getter(fn system_encryption_config)]
	pub type SystemEncryptionConfig<T: Config> = StorageValue<
		_,
		SystemEncryptionSettings,
		ValueQuery,
	>;

	// --- Audit Logging ---

	/// Audit Logs: Comprehensive activity logs
	#[pallet::storage]
	#[pallet::getter(fn audit_logs)]
	pub type AuditLogs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Log ID
		AuditLogEntry<T>,
		OptionQuery,
	>;

	/// Account Audit Logs: Maps account to their audit log IDs
	#[pallet::storage]
	#[pallet::getter(fn account_audit_logs)]
	pub type AccountAuditLogs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxAuditLogsPerAccount>,
		ValueQuery,
	>;

	/// Audit Log Counter: Total number of audit logs
	#[pallet::storage]
	#[pallet::getter(fn audit_log_counter)]
	pub type AuditLogCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Audit Configuration
	#[pallet::storage]
	#[pallet::getter(fn audit_config)]
	pub type AuditConfig<T: Config> = StorageValue<
		_,
		AuditConfiguration,
		ValueQuery,
	>;

	// --- Backup & Recovery ---

	/// Backup Snapshots: Stored backup information
	#[pallet::storage]
	#[pallet::getter(fn backup_snapshots)]
	pub type BackupSnapshots<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Snapshot ID
		BackupSnapshot<T>,
		OptionQuery,
	>;

	/// Backup Schedule: Automated backup configuration
	#[pallet::storage]
	#[pallet::getter(fn backup_schedule)]
	pub type BackupSchedule<T: Config> = StorageValue<
		_,
		BackupScheduleConfig<T>,
		OptionQuery,
	>;

	/// Last Backup: Timestamp of last backup
	#[pallet::storage]
	#[pallet::getter(fn last_backup)]
	pub type LastBackup<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Recovery Points: Point-in-time recovery markers
	#[pallet::storage]
	#[pallet::getter(fn recovery_points)]
	pub type RecoveryPoints<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		RecoveryPointInfo<T>,
		OptionQuery,
	>;

	// --- Advanced Authentication ---

	/// MFA Settings: Multi-factor authentication configuration
	#[pallet::storage]
	#[pallet::getter(fn mfa_settings)]
	pub type MfaSettings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		MfaConfiguration<T>,
		OptionQuery,
	>;

	/// MFA Devices: Registered MFA devices per account
	#[pallet::storage]
	#[pallet::getter(fn mfa_devices)]
	pub type MfaDevices<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<MfaDevice<T>, T::MaxMfaDevices>,
		ValueQuery,
	>;

	/// OAuth Providers: Registered OAuth providers
	#[pallet::storage]
	#[pallet::getter(fn oauth_providers)]
	pub type OAuthProviders<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OAuthProvider,
		OAuthProviderConfig,
		OptionQuery,
	>;

	/// OAuth Connections: User OAuth connections
	#[pallet::storage]
	#[pallet::getter(fn oauth_connections)]
	pub type OAuthConnections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<OAuthConnection<T>, T::MaxOAuthProviders>,
		ValueQuery,
	>;

	/// Active Sessions: Track active user sessions
	#[pallet::storage]
	#[pallet::getter(fn active_sessions)]
	pub type ActiveSessions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Session ID
		SessionInfo<T>,
		OptionQuery,
	>;

	/// Session Blacklist: Revoked sessions
	#[pallet::storage]
	#[pallet::getter(fn session_blacklist)]
	pub type SessionBlacklist<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash, // Session ID
		BlockNumberFor<T>, // Revoked at
		OptionQuery,
	>;

	/// IP Whitelist: Allowed IP addresses
	#[pallet::storage]
	#[pallet::getter(fn ip_whitelist)]
	pub type IpWhitelist<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<IpAddress<T>, ConstU32<100>>,
		ValueQuery,
	>;

	/// IP Blacklist: Blocked IP addresses
	#[pallet::storage]
	#[pallet::getter(fn ip_blacklist)]
	pub type IpBlacklist<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		IpAddress<T>,
		IpBanInfo<T>,
		OptionQuery,
	>;

	/// Failed Login Attempts: Track brute force attempts
	#[pallet::storage]
	#[pallet::getter(fn failed_login_attempts)]
	pub type FailedLoginAttempts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		LoginAttemptInfo<T>,
		ValueQuery,
	>;

	/// Security Policies: System-wide security settings
	#[pallet::storage]
	#[pallet::getter(fn security_policies)]
	pub type SecurityPolicies<T: Config> = StorageValue<
		_,
		SecurityPolicyConfig<T>,
		ValueQuery,
	>;

	// ===== Events =====

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Encryption Events
		/// Encryption key created [account]
		EncryptionKeyCreated { account: T::AccountId },
		/// Encryption key rotated [account, old_key_hash, new_key_hash]
		EncryptionKeyRotated { account: T::AccountId, old_key_hash: T::Hash, new_key_hash: T::Hash },
		/// Data encrypted [data_hash, encryption_type]
		DataEncrypted { data_hash: T::Hash, encryption_type: EncryptionType },
		/// Data decrypted [data_hash, accessor]
		DataDecrypted { data_hash: T::Hash, accessor: T::AccountId },
		/// System encryption enabled
		SystemEncryptionEnabled,

		// Audit Log Events
		/// Audit log created [log_id, action_type, account]
		AuditLogCreated { log_id: T::Hash, action_type: AuditAction, account: T::AccountId },
		/// Audit logs exported [export_id, log_count]
		AuditLogsExported { export_id: T::Hash, log_count: u32 },
		/// Suspicious activity detected [account, activity_type]
		SuspiciousActivityDetected { account: T::AccountId, activity_type: Vec<u8> },

		// Backup Events
		/// Backup created [snapshot_id, backup_type]
		BackupCreated { snapshot_id: T::Hash, backup_type: BackupType },
		/// Backup schedule updated [interval]
		BackupScheduleUpdated { interval: BlockNumberFor<T> },
		/// Recovery point created [block_number]
		RecoveryPointCreated { block_number: BlockNumberFor<T> },
		/// Recovery initiated [snapshot_id, initiator]
		RecoveryInitiated { snapshot_id: T::Hash, initiator: T::AccountId },
		/// Recovery completed [snapshot_id]
		RecoveryCompleted { snapshot_id: T::Hash },

		// MFA Events
		/// MFA enabled [account, method]
		MfaEnabled { account: T::AccountId, method: MfaMethod },
		/// MFA disabled [account]
		MfaDisabled { account: T::AccountId },
		/// MFA device registered [account, device_id, device_type]
		MfaDeviceRegistered { account: T::AccountId, device_id: Vec<u8>, device_type: MfaMethod },
		/// MFA verification succeeded [account]
		MfaVerificationSucceeded { account: T::AccountId },
		/// MFA verification failed [account]
		MfaVerificationFailed { account: T::AccountId },

		// OAuth Events
		/// OAuth provider registered [provider, client_id]
		OAuthProviderRegistered { provider: OAuthProvider, client_id: Vec<u8> },
		/// OAuth connection established [account, provider]
		OAuthConnected { account: T::AccountId, provider: OAuthProvider },
		/// OAuth connection removed [account, provider]
		OAuthDisconnected { account: T::AccountId, provider: OAuthProvider },

		// Session Events
		/// Session created [session_id, account]
		SessionCreated { session_id: T::Hash, account: T::AccountId },
		/// Session expired [session_id]
		SessionExpired { session_id: T::Hash },
		/// Session revoked [session_id, reason]
		SessionRevoked { session_id: T::Hash, reason: Vec<u8> },
		/// Session activity recorded [session_id]
		SessionActivity { session_id: T::Hash },

		// Security Events
		/// IP whitelisted [account, ip]
		IpWhitelisted { account: T::AccountId, ip: Vec<u8> },
		/// IP blacklisted [ip, reason]
		IpBlacklisted { ip: Vec<u8>, reason: Vec<u8> },
		/// Failed login attempt [account, attempts]
		FailedLoginAttempt { account: T::AccountId, attempts: u32 },
		/// Account locked [account, duration]
		AccountLocked { account: T::AccountId, duration: BlockNumberFor<T> },
		/// Security policy updated [policy_type]
		SecurityPolicyUpdated { policy_type: Vec<u8> },
	}

	// ===== Errors =====

	#[pallet::error]
	pub enum Error<T> {
		// Encryption Errors
		EncryptionKeyAlreadyExists,
		EncryptionKeyNotFound,
		InvalidEncryptionKey,
		DecryptionFailed,
		EncryptionDisabled,

		// Audit Log Errors
		AuditLogNotFound,
		MaxAuditLogsReached,
		InvalidAuditAction,

		// Backup Errors
		BackupNotFound,
		BackupFailed,
		InvalidBackupSchedule,
		RecoveryPointNotFound,
		RecoveryFailed,
		MaxBackupsReached,

		// MFA Errors
		MfaAlreadyEnabled,
		MfaNotEnabled,
		MfaDeviceNotFound,
		InvalidMfaCode,
		MaxMfaDevicesReached,
		MfaVerificationFailed,

		// OAuth Errors
		OAuthProviderNotFound,
		OAuthProviderAlreadyExists,
		OAuthConnectionFailed,
		InvalidOAuthToken,

		// Session Errors
		SessionNotFound,
		SessionExpired,
		SessionRevoked,
		InvalidSessionToken,
		TooManySessions,

		// Security Errors
		IpNotWhitelisted,
		IpBlacklisted,
		AccountLocked,
		TooManyFailedAttempts,
		PasswordTooWeak,
		Unauthorized,
	}

	// ===== Extrinsics =====

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// --- Encryption Management ---

		/// Create encryption key for account
		///
		/// Parameters:
		/// - `origin`: The account creating the encryption key
		/// - `public_key`: Public key for encryption
		/// - `key_type`: Type of encryption algorithm
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_encryption_key())]
		pub fn create_encryption_key(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
			key_type: EncryptionType,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				!EncryptionKeys::<T>::contains_key(&who),
				Error::<T>::EncryptionKeyAlreadyExists
			);

			let key_hash = T::Hashing::hash_of(&public_key);

			let key_info = EncryptionKeyInfo {
				public_key: public_key.try_into().map_err(|_| Error::<T>::InvalidEncryptionKey)?,
				key_type,
				created_at: frame_system::Pallet::<T>::block_number(),
				last_rotated: None,
				rotation_count: 0,
			};

			EncryptionKeys::<T>::insert(&who, key_info);

			// Log this action
			Self::log_audit_action(&who, AuditAction::EncryptionKeyCreated, None)?;

			Self::deposit_event(Event::EncryptionKeyCreated { account: who });

			Ok(())
		}

		/// Rotate encryption key
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::rotate_encryption_key())]
		pub fn rotate_encryption_key(
			origin: OriginFor<T>,
			new_public_key: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let old_key_hash = EncryptionKeys::<T>::get(&who)
				.map(|k| T::Hashing::hash_of(&k.public_key))
				.ok_or(Error::<T>::EncryptionKeyNotFound)?;

			let new_key_hash = T::Hashing::hash_of(&new_public_key);

			EncryptionKeys::<T>::try_mutate(&who, |key| -> DispatchResult {
				let k = key.as_mut().ok_or(Error::<T>::EncryptionKeyNotFound)?;
				k.public_key = new_public_key.try_into().map_err(|_| Error::<T>::InvalidEncryptionKey)?;
				k.last_rotated = Some(frame_system::Pallet::<T>::block_number());
				k.rotation_count = k.rotation_count.saturating_add(1);
				Ok(())
			})?;

			Self::log_audit_action(&who, AuditAction::EncryptionKeyRotated, None)?;

			Self::deposit_event(Event::EncryptionKeyRotated {
				account: who,
				old_key_hash,
				new_key_hash,
			});

			Ok(())
		}

		/// Mark data as encrypted
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::encrypt_data())]
		pub fn encrypt_data(
			origin: OriginFor<T>,
			data_hash: T::Hash,
			encryption_type: EncryptionType,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let encryption_info = DataEncryptionInfo {
				encrypted_by: who.clone(),
				encryption_type: encryption_type.clone(),
				encrypted_at: frame_system::Pallet::<T>::block_number(),
				access_count: 0,
			};

			EncryptionStatus::<T>::insert(&data_hash, encryption_info);

			Self::log_audit_action(&who, AuditAction::DataEncrypted, Some(data_hash))?;

			Self::deposit_event(Event::DataEncrypted {
				data_hash,
				encryption_type,
			});

			Ok(())
		}

		// --- Audit Logging ---

		/// Manually create audit log entry
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::create_audit_log())]
		pub fn create_audit_log(
			origin: OriginFor<T>,
			action: AuditAction,
			details: Vec<u8>,
			severity: AuditSeverity,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::log_audit_action_detailed(&who, action.clone(), None, details, severity)?;

			Ok(())
		}

		/// Export audit logs for analysis
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::export_audit_logs())]
		pub fn export_audit_logs(
			origin: OriginFor<T>,
			start_block: BlockNumberFor<T>,
			end_block: BlockNumberFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Generate export ID
			let export_id = T::Hashing::hash_of(&(who.clone(), start_block, end_block));

			// Count logs (simplified - in production would filter by block range)
			let log_count = AuditLogCounter::<T>::get() as u32;

			Self::log_audit_action(&who, AuditAction::AuditLogsExported, Some(export_id))?;

			Self::deposit_event(Event::AuditLogsExported {
				export_id,
				log_count,
			});

			Ok(())
		}

		// --- Backup & Recovery ---

		/// Create backup snapshot
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::create_backup())]
		pub fn create_backup(
			origin: OriginFor<T>,
			backup_type: BackupType,
			description: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let snapshot_id = T::Hashing::hash_of(&(
				who.clone(),
				frame_system::Pallet::<T>::block_number(),
				backup_type.clone(),
			));

			let snapshot = BackupSnapshot {
				created_by: who.clone(),
				created_at: frame_system::Pallet::<T>::block_number(),
				backup_type: backup_type.clone(),
				description: description.try_into().map_err(|_| Error::<T>::BackupFailed)?,
				data_hash: snapshot_id,
				size_bytes: 0, // Would be calculated in real implementation
				is_verified: false,
			};

			BackupSnapshots::<T>::insert(&snapshot_id, snapshot);
			LastBackup::<T>::put(frame_system::Pallet::<T>::block_number());

			Self::log_audit_action(&who, AuditAction::BackupCreated, Some(snapshot_id))?;

			Self::deposit_event(Event::BackupCreated {
				snapshot_id,
				backup_type,
			});

			Ok(())
		}

		/// Configure automated backup schedule
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::configure_backup_schedule())]
		pub fn configure_backup_schedule(
			origin: OriginFor<T>,
			interval: BlockNumberFor<T>,
			backup_type: BackupType,
			retention_count: u32,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let schedule = BackupScheduleConfig {
				interval,
				backup_type,
				retention_count,
				last_backup: LastBackup::<T>::get(),
				is_enabled: true,
			};

			BackupSchedule::<T>::put(schedule);

			Self::deposit_event(Event::BackupScheduleUpdated { interval });

			Ok(())
		}

		/// Create recovery point
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::create_recovery_point())]
		pub fn create_recovery_point(
			origin: OriginFor<T>,
			description: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let block_number = frame_system::Pallet::<T>::block_number();

			let recovery_point = RecoveryPointInfo {
				created_by: who.clone(),
				description: description.try_into().map_err(|_| Error::<T>::BackupFailed)?,
				state_hash: T::Hashing::hash_of(&block_number),
			};

			RecoveryPoints::<T>::insert(&block_number, recovery_point);

			Self::log_audit_action(&who, AuditAction::RecoveryPointCreated, None)?;

			Self::deposit_event(Event::RecoveryPointCreated { block_number });

			Ok(())
		}

		// --- MFA Management ---

		/// Enable multi-factor authentication
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::enable_mfa())]
		pub fn enable_mfa(
			origin: OriginFor<T>,
			method: MfaMethod,
			backup_codes: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				!MfaSettings::<T>::contains_key(&who),
				Error::<T>::MfaAlreadyEnabled
			);

			// Convert Vec<Vec<u8>> to BoundedVec<BoundedVec<u8, ...>, ...>
			let bounded_codes: BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<10>> = backup_codes
				.into_iter()
				.map(|code| code.try_into().map_err(|_| Error::<T>::MaxMfaDevicesReached))
				.collect::<Result<Vec<_>, _>>()?
				.try_into()
				.map_err(|_| Error::<T>::MaxMfaDevicesReached)?;

			let mfa_config = MfaConfiguration {
				primary_method: method.clone(),
				backup_codes: bounded_codes,
				created_at: frame_system::Pallet::<T>::block_number(),
				last_verified: None,
				is_required: true,
			};

			MfaSettings::<T>::insert(&who, mfa_config);

			Self::log_audit_action(&who, AuditAction::MfaEnabled, None)?;

			Self::deposit_event(Event::MfaEnabled {
				account: who,
				method,
			});

			Ok(())
		}

		/// Register MFA device
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::register_mfa_device())]
		pub fn register_mfa_device(
			origin: OriginFor<T>,
			device_id: Vec<u8>,
			device_type: MfaMethod,
			device_name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let device = MfaDevice {
				device_id: device_id.clone(),
				device_type: device_type.clone(),
				device_name,
				registered_at: frame_system::Pallet::<T>::block_number(),
				last_used: None,
				is_trusted: false,
			};

			MfaDevices::<T>::try_mutate(&who, |devices| {
				devices.try_push(device)
					.map_err(|_| Error::<T>::MaxMfaDevicesReached)
			})?;

			Self::log_audit_action(&who, AuditAction::MfaDeviceRegistered, None)?;

			Self::deposit_event(Event::MfaDeviceRegistered {
				account: who,
				device_id,
				device_type,
			});

			Ok(())
		}

		/// Verify MFA code
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::verify_mfa())]
		pub fn verify_mfa(
			origin: OriginFor<T>,
			code: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// In production, this would verify TOTP/SMS/backup code
			// For now, we just update the last verified timestamp

			MfaSettings::<T>::try_mutate(&who, |settings| -> DispatchResult {
				let s = settings.as_mut().ok_or(Error::<T>::MfaNotEnabled)?;
				s.last_verified = Some(frame_system::Pallet::<T>::block_number());
				Ok(())
			})?;

			Self::log_audit_action(&who, AuditAction::MfaVerified, None)?;

			Self::deposit_event(Event::MfaVerificationSucceeded { account: who });

			Ok(())
		}

		// --- OAuth Management ---

		/// Register OAuth provider
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::register_oauth_provider())]
		pub fn register_oauth_provider(
			origin: OriginFor<T>,
			provider: OAuthProvider,
			client_id: Vec<u8>,
			client_secret: Vec<u8>,
			redirect_uri: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// TODO: Add admin permission check

			ensure!(
				!OAuthProviders::<T>::contains_key(&provider),
				Error::<T>::OAuthProviderAlreadyExists
			);

			let config = OAuthProviderConfig {
				client_id: client_id.clone(),
				client_secret,
				redirect_uri,
				scopes: vec![],
				is_enabled: true,
			};

			OAuthProviders::<T>::insert(&provider, config);

			Self::deposit_event(Event::OAuthProviderRegistered {
				provider,
				client_id,
			});

			Ok(())
		}

		/// Connect OAuth account
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::connect_oauth())]
		pub fn connect_oauth(
			origin: OriginFor<T>,
			provider: OAuthProvider,
			oauth_user_id: Vec<u8>,
			access_token: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				OAuthProviders::<T>::contains_key(&provider),
				Error::<T>::OAuthProviderNotFound
			);

			let connection = OAuthConnection {
				provider: provider.clone(),
				oauth_user_id,
				connected_at: frame_system::Pallet::<T>::block_number(),
				last_used: None,
			};

			OAuthConnections::<T>::try_mutate(&who, |connections| {
				// Remove existing connection to same provider if any
				connections.retain(|c| c.provider != provider);
				connections.try_push(connection)
					.map_err(|_| Error::<T>::OAuthConnectionFailed)
			})?;

			Self::log_audit_action(&who, AuditAction::OAuthConnected, None)?;

			Self::deposit_event(Event::OAuthConnected {
				account: who,
				provider,
			});

			Ok(())
		}

		// --- Session Management ---

		/// Create new session
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::create_session())]
		pub fn create_session(
			origin: OriginFor<T>,
			ip_address: Vec<u8>,
			user_agent: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if account is locked
			let failed_attempts = FailedLoginAttempts::<T>::get(&who);
			ensure!(
				!failed_attempts.is_locked,
				Error::<T>::AccountLocked
			);

			let session_id = T::Hashing::hash_of(&(
				who.clone(),
				frame_system::Pallet::<T>::block_number(),
				&ip_address,
			));

			let session = SessionInfo {
				account: who.clone(),
				created_at: frame_system::Pallet::<T>::block_number(),
				last_activity: frame_system::Pallet::<T>::block_number(),
				ip_address: ip_address.try_into().map_err(|_| Error::<T>::InvalidSessionToken)?,
				user_agent: user_agent.try_into().map_err(|_| Error::<T>::InvalidSessionToken)?,
				is_mfa_verified: false,
			};

			ActiveSessions::<T>::insert(&session_id, session);

			Self::log_audit_action(&who, AuditAction::SessionCreated, Some(session_id))?;

			Self::deposit_event(Event::SessionCreated {
				session_id,
				account: who,
			});

			Ok(())
		}

		/// Revoke session
		#[pallet::call_index(14)]
		#[pallet::weight(T::WeightInfo::revoke_session())]
		pub fn revoke_session(
			origin: OriginFor<T>,
			session_id: T::Hash,
			reason: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Verify session belongs to user
			let session = ActiveSessions::<T>::get(&session_id)
				.ok_or(Error::<T>::SessionNotFound)?;
			ensure!(session.account == who, Error::<T>::Unauthorized);

			ActiveSessions::<T>::remove(&session_id);
			SessionBlacklist::<T>::insert(&session_id, frame_system::Pallet::<T>::block_number());

			Self::log_audit_action(&who, AuditAction::SessionRevoked, Some(session_id))?;

			Self::deposit_event(Event::SessionRevoked {
				session_id,
				reason,
			});

			Ok(())
		}

		// --- IP Management ---

		/// Add IP to whitelist
		#[pallet::call_index(15)]
		#[pallet::weight(T::WeightInfo::whitelist_ip())]
		pub fn whitelist_ip(
			origin: OriginFor<T>,
			ip_address: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let ip = IpAddress {
				address: ip_address.clone(),
				added_at: frame_system::Pallet::<T>::block_number(),
			};

			IpWhitelist::<T>::try_mutate(&who, |ips| {
				ips.try_push(ip)
					.map_err(|_| Error::<T>::TooManySessions)
			})?;

			Self::log_audit_action(&who, AuditAction::IpWhitelisted, None)?;

			Self::deposit_event(Event::IpWhitelisted {
				account: who,
				ip: ip_address,
			});

			Ok(())
		}

		/// Block IP address
		#[pallet::call_index(16)]
		#[pallet::weight(T::WeightInfo::blacklist_ip())]
		pub fn blacklist_ip(
			origin: OriginFor<T>,
			ip_address: Vec<u8>,
			reason: Vec<u8>,
			duration: Option<BlockNumberFor<T>>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// TODO: Add admin permission check

			let ip = IpAddress {
				address: ip_address.clone(),
				added_at: frame_system::Pallet::<T>::block_number(),
			};

			let ban_info = IpBanInfo {
				banned_at: frame_system::Pallet::<T>::block_number(),
				reason: reason.clone().try_into().map_err(|_| Error::<T>::Unauthorized)?,
				expires_at: duration.map(|d| frame_system::Pallet::<T>::block_number().saturating_add(d)),
			};

			IpBlacklist::<T>::insert(&ip, ban_info);

			Self::deposit_event(Event::IpBlacklisted {
				ip: ip_address,
				reason,
			});

			Ok(())
		}

		/// Record failed login attempt
		#[pallet::call_index(17)]
		#[pallet::weight(T::WeightInfo::record_failed_login())]
		pub fn record_failed_login(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let max_attempts = SecurityPolicies::<T>::get().max_failed_login_attempts;

			FailedLoginAttempts::<T>::mutate(&account, |attempts| {
				attempts.attempt_count = attempts.attempt_count.saturating_add(1);
				attempts.last_attempt = frame_system::Pallet::<T>::block_number();

				if attempts.attempt_count >= max_attempts {
					attempts.is_locked = true;
					attempts.locked_until = Some(
						frame_system::Pallet::<T>::block_number()
							.saturating_add(SecurityPolicies::<T>::get().lockout_duration)
					);
				}
			});

			let current_attempts = FailedLoginAttempts::<T>::get(&account);

			Self::log_audit_action(&account, AuditAction::FailedLoginAttempt, None)?;

			Self::deposit_event(Event::FailedLoginAttempt {
				account: account.clone(),
				attempts: current_attempts.attempt_count,
			});

			if current_attempts.is_locked {
				Self::deposit_event(Event::AccountLocked {
					account,
					duration: SecurityPolicies::<T>::get().lockout_duration,
				});
			}

			Ok(())
		}

		/// Update security policies
		#[pallet::call_index(18)]
		#[pallet::weight(T::WeightInfo::update_security_policy())]
		pub fn update_security_policy(
			origin: OriginFor<T>,
			policy: SecurityPolicyConfig<T>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// TODO: Add admin permission check

			SecurityPolicies::<T>::put(policy);

			Self::deposit_event(Event::SecurityPolicyUpdated {
				policy_type: b"system_wide".to_vec(),
			});

			Ok(())
		}
	}

	// ===== Helper Functions =====

	impl<T: Config> Pallet<T> {
		/// Log audit action with default parameters
		pub fn log_audit_action(
			account: &T::AccountId,
			action: AuditAction,
			related_entity: Option<T::Hash>,
		) -> DispatchResult {
			Self::log_audit_action_detailed(
				account,
				action,
				related_entity,
				vec![],
				AuditSeverity::Info,
			)
		}

		/// Log audit action with full details
		pub fn log_audit_action_detailed(
			account: &T::AccountId,
			action: AuditAction,
			related_entity: Option<T::Hash>,
			details: Vec<u8>,
			severity: AuditSeverity,
		) -> DispatchResult {
			let log_id = T::Hashing::hash_of(&(
				account.clone(),
				frame_system::Pallet::<T>::block_number(),
				AuditLogCounter::<T>::get(),
			));

			let log_entry = AuditLogEntry {
				account: account.clone(),
				action: action.clone(),
				timestamp: frame_system::Pallet::<T>::block_number(),
				ip_address: None,
				details: details.try_into().map_err(|_| Error::<T>::InvalidAuditAction)?,
				related_entity,
				severity,
			};

			AuditLogs::<T>::insert(&log_id, log_entry);

			AccountAuditLogs::<T>::try_mutate(account, |logs| {
				logs.try_push(log_id)
					.map_err(|_| Error::<T>::MaxAuditLogsReached)
			})?;

			AuditLogCounter::<T>::mutate(|count| *count = count.saturating_add(1));

			Self::deposit_event(Event::AuditLogCreated {
				log_id,
				action_type: action,
				account: account.clone(),
			});

			Ok(())
		}

		/// Check if session is valid
		pub fn is_session_valid(session_id: &T::Hash) -> bool {
			if SessionBlacklist::<T>::contains_key(session_id) {
				return false;
			}

			if let Some(session) = ActiveSessions::<T>::get(session_id) {
				let timeout = T::SessionTimeout::get();
				let current_block = frame_system::Pallet::<T>::block_number();
				let elapsed = current_block.saturating_sub(session.last_activity);

				elapsed < timeout
			} else {
				false
			}
		}

		/// Check if IP is allowed
		pub fn is_ip_allowed(ip: &IpAddress<T>) -> bool {
			!IpBlacklist::<T>::contains_key(ip)
		}
	}
}

// ===== Type Definitions =====

use codec::{Decode, Encode};
use frame::prelude::*;
use scale_info::TypeInfo;

// --- Encryption Types ---

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct EncryptionKeyInfo<T: Config> {
	pub public_key: BoundedVec<u8, T::MaxEncryptionKeyLength>,
	pub key_type: EncryptionType,
	pub created_at: BlockNumberFor<T>,
	pub last_rotated: Option<BlockNumberFor<T>>,
	pub rotation_count: u32,
}

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EncryptionType {
	AES256,
	RSA2048,
	RSA4096,
	ChaCha20,
	ECIES,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct DataEncryptionInfo<T: Config> {
	pub encrypted_by: T::AccountId,
	pub encryption_type: EncryptionType,
	pub encrypted_at: BlockNumberFor<T>,
	pub access_count: u32,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SystemEncryptionSettings {
	pub is_enabled: bool,
	pub default_algorithm: EncryptionType,
	pub enforce_encryption: bool,
	pub key_rotation_interval: u32,
}

impl Default for SystemEncryptionSettings {
	fn default() -> Self {
		Self {
			is_enabled: true,
			default_algorithm: EncryptionType::AES256,
			enforce_encryption: false,
			key_rotation_interval: 90, // days
		}
	}
}

// --- Audit Log Types ---

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct AuditLogEntry<T: Config> {
	pub account: T::AccountId,
	pub action: AuditAction,
	pub timestamp: BlockNumberFor<T>,
	pub ip_address: Option<BoundedVec<u8, ConstU32<45>>>,
	pub details: BoundedVec<u8, ConstU32<512>>,
	pub related_entity: Option<T::Hash>,
	pub severity: AuditSeverity,
}

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AuditAction {
	// User actions
	UserLogin,
	UserLogout,
	UserRegistered,
	UserUpdated,
	PasswordChanged,
	FailedLoginAttempt,

	// Encryption actions
	EncryptionKeyCreated,
	EncryptionKeyRotated,
	DataEncrypted,
	DataDecrypted,

	// MFA actions
	MfaEnabled,
	MfaDisabled,
	MfaDeviceRegistered,
	MfaVerified,

	// OAuth actions
	OAuthConnected,
	OAuthDisconnected,

	// Session actions
	SessionCreated,
	SessionRevoked,

	// Security actions
	IpWhitelisted,
	IpBlacklisted,

	// Backup actions
	BackupCreated,
	RecoveryPointCreated,
	RecoveryInitiated,

	// System actions
	AuditLogsExported,
	SecurityPolicyUpdated,

	// Other
	Custom,
}

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AuditSeverity {
	Info,
	Warning,
	Error,
	Critical,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AuditConfiguration {
	pub is_enabled: bool,
	pub retention_period: u32, // blocks
	pub log_all_actions: bool,
	pub alert_on_suspicious: bool,
}

impl Default for AuditConfiguration {
	fn default() -> Self {
		Self {
			is_enabled: true,
			retention_period: 2_592_000, // ~180 days assuming 6s blocks
			log_all_actions: true,
			alert_on_suspicious: true,
		}
	}
}

// --- Backup Types ---

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct BackupSnapshot<T: Config> {
	pub created_by: T::AccountId,
	pub created_at: BlockNumberFor<T>,
	pub backup_type: BackupType,
	pub description: BoundedVec<u8, ConstU32<256>>,
	pub data_hash: T::Hash,
	pub size_bytes: u64,
	pub is_verified: bool,
}

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum BackupType {
	Full,
	Incremental,
	Differential,
	StateSnapshot,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct BackupScheduleConfig<T: Config> {
	pub interval: BlockNumberFor<T>,
	pub backup_type: BackupType,
	pub retention_count: u32,
	pub last_backup: BlockNumberFor<T>,
	pub is_enabled: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RecoveryPointInfo<T: Config> {
	pub created_by: T::AccountId,
	pub description: BoundedVec<u8, ConstU32<256>>,
	pub state_hash: T::Hash,
}

// --- MFA Types ---

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct MfaConfiguration<T: Config> {
	pub primary_method: MfaMethod,
	pub backup_codes: BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<10>>,
	pub created_at: BlockNumberFor<T>,
	pub last_verified: Option<BlockNumberFor<T>>,
	pub is_required: bool,
}

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MfaMethod {
	TOTP,        // Time-based One-Time Password (Google Authenticator)
	SMS,         // SMS verification
	Email,       // Email verification
	Hardware,    // Hardware security key (YubiKey, etc.)
	Biometric,   // Fingerprint, Face ID, etc.
	BackupCode,  // One-time backup codes
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct MfaDevice<T: Config> {
	pub device_id: Vec<u8>,
	pub device_type: MfaMethod,
	pub device_name: Vec<u8>,
	pub registered_at: BlockNumberFor<T>,
	pub last_used: Option<BlockNumberFor<T>>,
	pub is_trusted: bool,
}

// --- OAuth Types ---

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OAuthProvider {
	Google,
	GitHub,
	Microsoft,
	Facebook,
	Twitter,
	LinkedIn,
	Custom,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OAuthProviderConfig {
	pub client_id: Vec<u8>,
	pub client_secret: Vec<u8>,
	pub redirect_uri: Vec<u8>,
	pub scopes: Vec<u8>,
	pub is_enabled: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct OAuthConnection<T: Config> {
	pub provider: OAuthProvider,
	pub oauth_user_id: Vec<u8>,
	pub connected_at: BlockNumberFor<T>,
	pub last_used: Option<BlockNumberFor<T>>,
}

// --- Session Types ---

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct SessionInfo<T: Config> {
	pub account: T::AccountId,
	pub created_at: BlockNumberFor<T>,
	pub last_activity: BlockNumberFor<T>,
	pub ip_address: BoundedVec<u8, ConstU32<45>>,
	pub user_agent: BoundedVec<u8, ConstU32<256>>,
	pub is_mfa_verified: bool,
}

// --- IP Management Types ---

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct IpAddress<T: Config> {
	pub address: Vec<u8>,
	pub added_at: BlockNumberFor<T>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct IpBanInfo<T: Config> {
	pub banned_at: BlockNumberFor<T>,
	pub reason: BoundedVec<u8, ConstU32<256>>,
	pub expires_at: Option<BlockNumberFor<T>>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct LoginAttemptInfo<T: Config> {
	pub attempt_count: u32,
	pub last_attempt: BlockNumberFor<T>,
	pub is_locked: bool,
	pub locked_until: Option<BlockNumberFor<T>>,
}

impl<T: Config> Default for LoginAttemptInfo<T> {
	fn default() -> Self {
		Self {
			attempt_count: 0,
			last_attempt: BlockNumberFor::<T>::default(),
			is_locked: false,
			locked_until: None,
		}
	}
}

// --- Security Policy Types ---

#[derive(Clone, Encode, Decode, frame::deps::codec::DecodeWithMemTracking, Eq, PartialEq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct SecurityPolicyConfig<T: Config> {
	pub require_mfa: bool,
	pub min_password_length: u8,
	pub password_complexity: bool,
	pub max_failed_login_attempts: u32,
	pub lockout_duration: BlockNumberFor<T>, // blocks
	pub session_timeout: BlockNumberFor<T>, // blocks
	pub require_ip_whitelist: bool,
	pub enable_audit_logging: bool,
	pub enforce_encryption: bool,
}

impl<T: Config> Default for SecurityPolicyConfig<T> {
	fn default() -> Self {
		Self {
			require_mfa: false,
			min_password_length: 12,
			password_complexity: true,
			max_failed_login_attempts: 5,
			lockout_duration: 600u32.into(), // ~1 hour with 6s blocks
			session_timeout: 14400u32.into(), // ~24 hours
			require_ip_whitelist: false,
			enable_audit_logging: true,
			enforce_encryption: false,
		}
	}
}
