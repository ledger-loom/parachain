//! Unit tests for company management pallet

use crate::{mock::*, Error, Event, MemberRole};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_company_works() {
	new_test_ext().execute_with(|| {
		let owner = AccountId32::from([1u8; 32]);
		let name = b"Acme Corp".to_vec();

		assert_ok!(CompanyManagement::create_company(
			RuntimeOrigin::signed(owner.clone()),
			name.clone()
		));

		assert_eq!(crate::NextCompanyId::<Test>::get(), 1);
		assert!(crate::Companies::<Test>::contains_key(0));
		assert_eq!(crate::UserCompany::<Test>::get(&owner), Some(0));
	});
}

#[test]
fn invite_and_accept_member_works() {
	new_test_ext().execute_with(|| {
		let owner = AccountId32::from([1u8; 32]);
		let member = AccountId32::from([2u8; 32]);

		assert_ok!(CompanyManagement::create_company(
			RuntimeOrigin::signed(owner.clone()),
			b"Acme Corp".to_vec()
		));

		assert_ok!(CompanyManagement::invite_member(
			RuntimeOrigin::signed(owner),
			0,
			member.clone(),
			MemberRole::Manager
		));

		assert_ok!(CompanyManagement::accept_invitation(
			RuntimeOrigin::signed(member.clone())
		));

		assert_eq!(crate::UserCompany::<Test>::get(&member), Some(0));
	});
}
