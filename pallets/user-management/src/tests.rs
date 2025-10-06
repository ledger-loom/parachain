//! Unit tests for user management pallet

use crate::{mock::*, Error, Event};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

type UserManagement = crate::Pallet<Test>;

#[test]
fn register_user_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let name = b"Alice".to_vec();
		let email_hash = [1u8; 32];

		// Register user
		assert_ok!(UserManagement::register_user(
			RuntimeOrigin::signed(user.clone()),
			name.clone(),
			email_hash
		));

		// Verify user exists
		assert!(crate::Users::<Test>::contains_key(&user));

		// Verify email mapping
		assert_eq!(crate::EmailToAccount::<Test>::get(email_hash), Some(user.clone()));

		// Verify user count
		assert_eq!(crate::UserCount::<Test>::get(), 1);

		// Verify event
		System::assert_last_event(
			Event::UserRegistered {
				account: user,
				email_hash,
			}
			.into(),
		);
	});
}

#[test]
fn register_user_fails_if_already_exists() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let name = b"Alice".to_vec();
		let email_hash = [1u8; 32];

		// Register user first time
		assert_ok!(UserManagement::register_user(
			RuntimeOrigin::signed(user.clone()),
			name.clone(),
			email_hash
		));

		// Try to register again
		assert_noop!(
			UserManagement::register_user(
				RuntimeOrigin::signed(user.clone()),
				name,
				email_hash
			),
			Error::<Test>::UserAlreadyExists
		);
	});
}

#[test]
fn register_user_fails_if_email_already_used() {
	new_test_ext().execute_with(|| {
		let user1 = AccountId32::from([1u8; 32]);
		let user2 = AccountId32::from([2u8; 32]);
		let email_hash = [1u8; 32];

		// Register first user
		assert_ok!(UserManagement::register_user(
			RuntimeOrigin::signed(user1),
			b"Alice".to_vec(),
			email_hash
		));

		// Try to register second user with same email
		assert_noop!(
			UserManagement::register_user(
				RuntimeOrigin::signed(user2),
				b"Bob".to_vec(),
				email_hash
			),
			Error::<Test>::EmailAlreadyRegistered
		);
	});
}

#[test]
fn update_profile_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let email_hash = [1u8; 32];

		// Register user
		assert_ok!(UserManagement::register_user(
			RuntimeOrigin::signed(user.clone()),
			b"Alice".to_vec(),
			email_hash
		));

		// Update profile
		let new_name = b"Alice Smith".to_vec();
		assert_ok!(UserManagement::update_profile(
			RuntimeOrigin::signed(user.clone()),
			Some(new_name.clone())
		));

		// Verify update
		let profile = crate::Users::<Test>::get(&user).unwrap();
		assert_eq!(profile.name.to_vec(), new_name);
	});
}

#[test]
fn submit_verification_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let email_hash = [1u8; 32];

		// Register user
		assert_ok!(UserManagement::register_user(
			RuntimeOrigin::signed(user.clone()),
			b"Alice".to_vec(),
			email_hash
		));

		// Submit verification
		let doc_hashes = vec![[1u8; 32], [2u8; 32]];
		assert_ok!(UserManagement::submit_verification(
			RuntimeOrigin::signed(user.clone()),
			crate::VerificationType::Identity,
			doc_hashes
		));

		// Verify request exists
		assert!(crate::VerificationRequests::<Test>::contains_key(&user));
	});
}

#[test]
fn approve_verification_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let email_hash = [1u8; 32];

		// Register user
		assert_ok!(UserManagement::register_user(
			RuntimeOrigin::signed(user.clone()),
			b"Alice".to_vec(),
			email_hash
		));

		// Submit verification
		assert_ok!(UserManagement::submit_verification(
			RuntimeOrigin::signed(user.clone()),
			crate::VerificationType::Identity,
			vec![[1u8; 32]]
		));

		// Approve verification (root)
		assert_ok!(UserManagement::approve_verification(RuntimeOrigin::root(), user.clone()));

		// Verify user is verified
		let profile = crate::Users::<Test>::get(&user).unwrap();
		assert!(profile.is_verified);

		// Verify request status
		let request = crate::VerificationRequests::<Test>::get(&user).unwrap();
		assert_eq!(request.status, crate::VerificationStatus::Approved);
	});
}
