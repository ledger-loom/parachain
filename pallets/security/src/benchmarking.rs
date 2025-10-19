//! Benchmarking setup for pallet-security

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_encryption_key() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), b"public_key".to_vec(), EncryptionType::AES256);
	}

	#[benchmark]
	fn rotate_encryption_key() {
		let caller: T::AccountId = whitelisted_caller();

		Pallet::<T>::create_encryption_key(
			RawOrigin::Signed(caller.clone()).into(),
			b"old_key".to_vec(),
			EncryptionType::AES256,
		).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), b"new_key".to_vec());
	}

	#[benchmark]
	fn encrypt_data() {
		let caller: T::AccountId = whitelisted_caller();
		let data_hash = T::Hashing::hash_of(&b"data".to_vec());

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), data_hash, EncryptionType::AES256);
	}

	#[benchmark]
	fn create_audit_log() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			AuditAction::UserLogin,
			b"details".to_vec(),
			AuditSeverity::Info
		);
	}

	#[benchmark]
	fn export_audit_logs() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), 0u32.into(), 100u32.into());
	}

	#[benchmark]
	fn create_backup() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			BackupType::Full,
			b"backup description".to_vec()
		);
	}

	#[benchmark]
	fn configure_backup_schedule() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), 1000u32.into(), BackupType::Incremental, 10);
	}

	#[benchmark]
	fn create_recovery_point() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), b"recovery point".to_vec());
	}

	#[benchmark]
	fn enable_mfa() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), MfaMethod::TOTP, vec![b"backup".to_vec()]);
	}

	#[benchmark]
	fn register_mfa_device() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			b"device_id".to_vec(),
			MfaMethod::Hardware,
			b"YubiKey".to_vec()
		);
	}

	#[benchmark]
	fn verify_mfa() {
		let caller: T::AccountId = whitelisted_caller();

		Pallet::<T>::enable_mfa(
			RawOrigin::Signed(caller.clone()).into(),
			MfaMethod::TOTP,
			vec![b"backup".to_vec()],
		).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), b"123456".to_vec());
	}

	#[benchmark]
	fn register_oauth_provider() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			OAuthProvider::Google,
			b"client_id".to_vec(),
			b"secret".to_vec(),
			b"https://callback".to_vec()
		);
	}

	#[benchmark]
	fn connect_oauth() {
		let caller: T::AccountId = whitelisted_caller();

		Pallet::<T>::register_oauth_provider(
			RawOrigin::Signed(caller.clone()).into(),
			OAuthProvider::Google,
			b"client_id".to_vec(),
			b"secret".to_vec(),
			b"https://callback".to_vec(),
		).unwrap();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			OAuthProvider::Google,
			b"oauth_user".to_vec(),
			b"token".to_vec()
		);
	}

	#[benchmark]
	fn create_session() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			b"192.168.1.1".to_vec(),
			b"Mozilla/5.0".to_vec()
		);
	}

	#[benchmark]
	fn revoke_session() {
		let caller: T::AccountId = whitelisted_caller();
		let session_id = T::Hashing::hash_of(&b"session".to_vec());

		// Setup: create session
		let session = SessionInfo {
			account: caller.clone(),
			created_at: 0u32.into(),
			last_activity: 0u32.into(),
			ip_address: b"192.168.1.1".to_vec().try_into().unwrap(),
			user_agent: b"Mozilla".to_vec().try_into().unwrap(),
			is_mfa_verified: false,
		};
		ActiveSessions::<T>::insert(&session_id, session);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), session_id, b"logout".to_vec());
	}

	#[benchmark]
	fn whitelist_ip() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), b"192.168.1.100".to_vec());
	}

	#[benchmark]
	fn blacklist_ip() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			b"10.0.0.1".to_vec(),
			b"suspicious".to_vec(),
			Some(1000u32.into())
		);
	}

	#[benchmark]
	fn record_failed_login() {
		let caller: T::AccountId = whitelisted_caller();
		let target: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), target);
	}

	#[benchmark]
	fn update_security_policy() {
		let caller: T::AccountId = whitelisted_caller();
		let policy = SecurityPolicyConfig::default();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), policy);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
