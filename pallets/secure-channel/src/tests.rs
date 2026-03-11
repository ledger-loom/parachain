use crate::{mock::*, Error, Event};
use frame::testing_prelude::*;

#[test]
fn register_core_public_key_works() {
	new_test_ext().execute_with(|| {
		let public_key = vec![1u8; 33]; // Mock 33-byte compressed P-256 key
		let core_account = 1u64;

		// Register core public key
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			public_key.clone()
		));

		// Verify storage
		assert_eq!(SecureChannel::is_core_registered(), true);
		assert_eq!(SecureChannel::core_public_key(), Some(public_key.clone().try_into().unwrap()));
		assert_eq!(SecureChannel::core_account(), Some(core_account));

		// Verify event
		System::assert_last_event(
			Event::CoreRegistered {
				public_key,
				registered_by: core_account,
			}
			.into(),
		);
	});
}

#[test]
fn register_core_public_key_fails_if_already_registered() {
	new_test_ext().execute_with(|| {
		let public_key = vec![1u8; 33];
		let core_account = 1u64;

		// Register first time
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			public_key.clone()
		));

		// Try to register again
		assert_noop!(
			SecureChannel::register_core_public_key(
				RuntimeOrigin::signed(2u64),
				vec![2u8; 33]
			),
			Error::<Test>::CoreAlreadyRegistered
		);
	});
}

#[test]
fn rotate_channel_key_works() {
	new_test_ext().execute_with(|| {
		let old_key = vec![1u8; 33];
		let new_key = vec![2u8; 33];
		let signature = vec![0u8; 64]; // Mock signature
		let core_account = 1u64;

		// Register first
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			old_key.clone()
		));

		// Rotate key
		assert_ok!(SecureChannel::rotate_channel_key(
			RuntimeOrigin::signed(core_account),
			new_key.clone(),
			signature
		));

		// Verify new key is stored
		assert_eq!(SecureChannel::core_public_key(), Some(new_key.clone().try_into().unwrap()));

		// Verify event
		System::assert_last_event(
			Event::KeyRotated {
				old_key,
				new_key,
				rotated_by: core_account,
			}
			.into(),
		);
	});
}

#[test]
fn rotate_channel_key_fails_if_not_authorized() {
	new_test_ext().execute_with(|| {
		let old_key = vec![1u8; 33];
		let new_key = vec![2u8; 33];
		let signature = vec![0u8; 64];
		let core_account = 1u64;
		let unauthorized = 2u64;

		// Register first
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			old_key.clone()
		));

		// Try to rotate with unauthorized account
		assert_noop!(
			SecureChannel::rotate_channel_key(
				RuntimeOrigin::signed(unauthorized),
				new_key,
				signature
			),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn verify_core_message_works() {
	new_test_ext().execute_with(|| {
		let public_key = vec![1u8; 33];
		let core_account = 1u64;

		// Register first
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			public_key
		));

		// Verify message with sequence 1
		assert_ok!(SecureChannel::verify_core_message(
			RuntimeOrigin::signed(core_account),
			1,
			vec![1, 2, 3],
			vec![0u8; 64]
		));

		assert_eq!(SecureChannel::last_sequence_number(), 1);

		// Verify message with sequence 2
		assert_ok!(SecureChannel::verify_core_message(
			RuntimeOrigin::signed(core_account),
			2,
			vec![4, 5, 6],
			vec![0u8; 64]
		));

		assert_eq!(SecureChannel::last_sequence_number(), 2);
	});
}

#[test]
fn verify_core_message_prevents_replay() {
	new_test_ext().execute_with(|| {
		let public_key = vec![1u8; 33];
		let core_account = 1u64;

		// Register first
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			public_key
		));

		// Verify message with sequence 1
		assert_ok!(SecureChannel::verify_core_message(
			RuntimeOrigin::signed(core_account),
			1,
			vec![1, 2, 3],
			vec![0u8; 64]
		));

		// Try to replay same sequence number
		assert_noop!(
			SecureChannel::verify_core_message(
				RuntimeOrigin::signed(core_account),
				1,
				vec![1, 2, 3],
				vec![0u8; 64]
			),
			Error::<Test>::InvalidSequenceNumber
		);

		// Try with lower sequence number
		assert_noop!(
			SecureChannel::verify_core_message(
				RuntimeOrigin::signed(core_account),
				0,
				vec![1, 2, 3],
				vec![0u8; 64]
			),
			Error::<Test>::InvalidSequenceNumber
		);
	});
}

#[test]
fn update_core_account_works() {
	new_test_ext().execute_with(|| {
		let public_key = vec![1u8; 33];
		let old_account = 1u64;
		let new_account = 2u64;

		// Register first
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(old_account),
			public_key
		));

		// Update core account (requires root)
		assert_ok!(SecureChannel::update_core_account(
			RuntimeOrigin::root(),
			new_account
		));

		// Verify new account
		assert_eq!(SecureChannel::core_account(), Some(new_account));

		// Verify event
		System::assert_last_event(
			Event::CoreAccountUpdated {
				old_account: Some(old_account),
				new_account,
			}
			.into(),
		);
	});
}

#[test]
fn key_rotation_history_works() {
	new_test_ext().execute_with(|| {
		let key1 = vec![1u8; 33];
		let key2 = vec![2u8; 33];
		let key3 = vec![3u8; 33];
		let signature = vec![0u8; 64];
		let core_account = 1u64;

		// Register initial key
		assert_ok!(SecureChannel::register_core_public_key(
			RuntimeOrigin::signed(core_account),
			key1.clone()
		));

		let history = SecureChannel::key_rotation_history();
		assert_eq!(history.len(), 1);

		// Rotate to key2
		assert_ok!(SecureChannel::rotate_channel_key(
			RuntimeOrigin::signed(core_account),
			key2.clone(),
			signature.clone()
		));

		let history = SecureChannel::key_rotation_history();
		assert_eq!(history.len(), 2);

		// Rotate to key3
		assert_ok!(SecureChannel::rotate_channel_key(
			RuntimeOrigin::signed(core_account),
			key3.clone(),
			signature
		));

		let history = SecureChannel::key_rotation_history();
		assert_eq!(history.len(), 3);
	});
}
