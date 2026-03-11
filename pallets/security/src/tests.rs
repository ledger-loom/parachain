use crate::{mock::*, Error, Event, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_encryption_key_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::create_encryption_key(
			RuntimeOrigin::signed(1),
			b"public_key_data".to_vec(),
			EncryptionType::AES256,
		));

		System::assert_last_event(
			Event::EncryptionKeyCreated { account: 1 }.into(),
		);

		assert!(EncryptionKeys::<Test>::contains_key(&1));
	});
}

#[test]
fn create_encryption_key_fails_if_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::create_encryption_key(
			RuntimeOrigin::signed(1),
			b"public_key_data".to_vec(),
			EncryptionType::AES256,
		));

		assert_noop!(
			Security::create_encryption_key(
				RuntimeOrigin::signed(1),
				b"another_key".to_vec(),
				EncryptionType::RSA2048,
			),
			Error::<Test>::EncryptionKeyAlreadyExists
		);
	});
}

#[test]
fn rotate_encryption_key_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::create_encryption_key(
			RuntimeOrigin::signed(1),
			b"old_key".to_vec(),
			EncryptionType::AES256,
		));

		assert_ok!(Security::rotate_encryption_key(
			RuntimeOrigin::signed(1),
			b"new_key".to_vec(),
		));

		let key_info = EncryptionKeys::<Test>::get(&1).unwrap();
		assert_eq!(key_info.rotation_count, 1);
	});
}

#[test]
fn create_audit_log_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::create_audit_log(
			RuntimeOrigin::signed(1),
			AuditAction::UserLogin,
			b"Login from IP 192.168.1.1".to_vec(),
			AuditSeverity::Info,
		));

		assert!(AuditLogCounter::<Test>::get() > 0);
	});
}

#[test]
fn create_backup_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::create_backup(
			RuntimeOrigin::signed(1),
			BackupType::Full,
			b"Weekly backup".to_vec(),
		));

		System::assert_has_event(
			Event::BackupCreated {
				snapshot_id: sp_core::H256::default(),
				backup_type: BackupType::Full,
			}
			.into(),
		);
	});
}

#[test]
fn enable_mfa_works() {
	new_test_ext().execute_with(|| {
		let backup_codes = vec![b"code1".to_vec(), b"code2".to_vec()];

		assert_ok!(Security::enable_mfa(
			RuntimeOrigin::signed(1),
			MfaMethod::TOTP,
			backup_codes,
		));

		System::assert_last_event(
			Event::MfaEnabled {
				account: 1,
				method: MfaMethod::TOTP,
			}
			.into(),
		);

		assert!(MfaSettings::<Test>::contains_key(&1));
	});
}

#[test]
fn enable_mfa_fails_if_already_enabled() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::enable_mfa(
			RuntimeOrigin::signed(1),
			MfaMethod::TOTP,
			vec![b"code".to_vec()],
		));

		assert_noop!(
			Security::enable_mfa(
				RuntimeOrigin::signed(1),
				MfaMethod::SMS,
				vec![b"code".to_vec()],
			),
			Error::<Test>::MfaAlreadyEnabled
		);
	});
}

#[test]
fn register_mfa_device_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::register_mfa_device(
			RuntimeOrigin::signed(1),
			b"device_123".to_vec(),
			MfaMethod::Hardware,
			b"YubiKey 5C".to_vec(),
		));

		let devices = MfaDevices::<Test>::get(&1);
		assert_eq!(devices.len(), 1);
	});
}

#[test]
fn verify_mfa_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::enable_mfa(
			RuntimeOrigin::signed(1),
			MfaMethod::TOTP,
			vec![b"backup".to_vec()],
		));

		assert_ok!(Security::verify_mfa(
			RuntimeOrigin::signed(1),
			b"123456".to_vec(),
		));

		System::assert_last_event(
			Event::MfaVerificationSucceeded { account: 1 }.into(),
		);
	});
}

#[test]
fn connect_oauth_works() {
	new_test_ext().execute_with(|| {
		// Register provider first
		assert_ok!(Security::register_oauth_provider(
			RuntimeOrigin::signed(1),
			OAuthProvider::Google,
			b"client_id_123".to_vec(),
			b"client_secret".to_vec(),
			b"https://example.com/callback".to_vec(),
		));

		// Connect OAuth
		assert_ok!(Security::connect_oauth(
			RuntimeOrigin::signed(2),
			OAuthProvider::Google,
			b"google_user_123".to_vec(),
			b"access_token".to_vec(),
		));

		let connections = OAuthConnections::<Test>::get(&2);
		assert_eq!(connections.len(), 1);
	});
}

#[test]
fn create_session_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::create_session(
			RuntimeOrigin::signed(1),
			b"192.168.1.1".to_vec(),
			b"Mozilla/5.0".to_vec(),
		));

		System::assert_has_event(
			Event::SessionCreated {
				session_id: sp_core::H256::default(),
				account: 1,
			}
			.into(),
		);
	});
}

#[test]
fn revoke_session_works() {
	new_test_ext().execute_with(|| {
		// Create session first
		assert_ok!(Security::create_session(
			RuntimeOrigin::signed(1),
			b"192.168.1.1".to_vec(),
			b"Mozilla/5.0".to_vec(),
		));

		let session_id = sp_core::H256::default();

		// Revoke it
		assert_ok!(Security::revoke_session(
			RuntimeOrigin::signed(1),
			session_id,
			b"User logged out".to_vec(),
		));

		assert!(SessionBlacklist::<Test>::contains_key(&session_id));
	});
}

#[test]
fn whitelist_ip_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::whitelist_ip(
			RuntimeOrigin::signed(1),
			b"192.168.1.100".to_vec(),
		));

		let ips = IpWhitelist::<Test>::get(&1);
		assert_eq!(ips.len(), 1);
	});
}

#[test]
fn blacklist_ip_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::blacklist_ip(
			RuntimeOrigin::signed(1),
			b"10.0.0.1".to_vec(),
			b"Suspicious activity".to_vec(),
			Some(1000),
		));

		System::assert_has_event(
			Event::IpBlacklisted {
				ip: b"10.0.0.1".to_vec(),
				reason: b"Suspicious activity".to_vec(),
			}
			.into(),
		);
	});
}

#[test]
fn record_failed_login_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Security::record_failed_login(
			RuntimeOrigin::signed(1),
			2,
		));

		let attempts = FailedLoginAttempts::<Test>::get(&2);
		assert_eq!(attempts.attempt_count, 1);
	});
}

#[test]
fn account_locks_after_max_attempts() {
	new_test_ext().execute_with(|| {
		// Record 5 failed attempts
		for _ in 0..5 {
			assert_ok!(Security::record_failed_login(
				RuntimeOrigin::signed(1),
				2,
			));
		}

		let attempts = FailedLoginAttempts::<Test>::get(&2);
		assert!(attempts.is_locked);
	});
}
