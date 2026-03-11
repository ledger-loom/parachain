//! Unit tests for product items pallet

use crate::{mock::*, Error, Event};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

// Test helper: Sample business UUID
fn sample_business_id() -> [u8; 16] {
	[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
}

#[test]
fn create_product_item_works() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();
		let name = b"Weight".to_vec();
		let unit = b"kg".to_vec();

		// Create product item
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			name.clone(),
			unit.clone(),
			None,
		));

		// Check storage
		assert_eq!(crate::NextItemId::<Test>::get(&business_id), 1);
		assert!(crate::ProductItems::<Test>::contains_key(&business_id, 0));

		// Check item details
		let item = crate::ProductItems::<Test>::get(&business_id, 0).unwrap();
		assert_eq!(item.item_id, 0);
		assert_eq!(item.business_id, business_id);
		assert_eq!(item.name.to_vec(), name);
		assert_eq!(item.unit.to_vec(), unit);
		assert_eq!(item.is_active, true);

		// Check name lookup
		let lowercase_name = b"weight".to_vec();
		assert!(crate::ProductItemNames::<Test>::contains_key(
			&business_id,
			&lowercase_name.try_into().unwrap()
		));

		// Check event
		System::assert_last_event(
			Event::ProductItemCreated {
				business_id,
				item_id: 0,
				name,
				unit,
			}
			.into(),
		);
	});
}

#[test]
fn create_product_item_with_description_works() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();
		let name = b"Volume".to_vec();
		let unit = b"liter".to_vec();
		let description = Some(b"Product volume in liters".to_vec());

		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			name.clone(),
			unit.clone(),
			description.clone(),
		));

		let item = crate::ProductItems::<Test>::get(&business_id, 0).unwrap();
		assert_eq!(
			item.description.map(|d| d.to_vec()),
			description
		);
	});
}

#[test]
fn create_duplicate_name_fails() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		// Create first item
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		// Try to create item with same name (case-insensitive)
		assert_noop!(
			ProductItems::create_product_item(
				RuntimeOrigin::signed(caller.clone()),
				business_id,
				b"WEIGHT".to_vec(), // Different case
				b"g".to_vec(),
				None,
			),
			Error::<Test>::ProductItemNameAlreadyExists
		);
	});
}

#[test]
fn create_empty_name_fails() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		assert_noop!(
			ProductItems::create_product_item(
				RuntimeOrigin::signed(caller.clone()),
				business_id,
				b"".to_vec(), // Empty name
				b"kg".to_vec(),
				None,
			),
			Error::<Test>::InvalidItemName
		);
	});
}

#[test]
fn create_empty_unit_fails() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		assert_noop!(
			ProductItems::create_product_item(
				RuntimeOrigin::signed(caller.clone()),
				business_id,
				b"Weight".to_vec(),
				b"".to_vec(), // Empty unit
				None,
			),
			Error::<Test>::InvalidUnit
		);
	});
}

#[test]
fn update_product_item_name_works() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		// Create item
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		// Update name
		assert_ok!(ProductItems::update_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			0,
			Some(b"Mass".to_vec()),
			None,
			None,
			None,
		));

		// Check updated name
		let item = crate::ProductItems::<Test>::get(&business_id, 0).unwrap();
		assert_eq!(item.name.to_vec(), b"Mass".to_vec());

		// Check event
		System::assert_last_event(
			Event::ProductItemUpdated {
				business_id,
				item_id: 0,
			}
			.into(),
		);
	});
}

#[test]
fn update_product_item_unit_works() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		// Create item
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		// Update unit
		assert_ok!(ProductItems::update_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			0,
			None,
			Some(b"g".to_vec()),
			None,
			None,
		));

		// Check updated unit
		let item = crate::ProductItems::<Test>::get(&business_id, 0).unwrap();
		assert_eq!(item.unit.to_vec(), b"g".to_vec());
	});
}

#[test]
fn update_nonexistent_item_fails() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		assert_noop!(
			ProductItems::update_product_item(
				RuntimeOrigin::signed(caller.clone()),
				business_id,
				999, // Non-existent item ID
				Some(b"Weight".to_vec()),
				None,
				None,
				None,
			),
			Error::<Test>::ProductItemNotFound
		);
	});
}

#[test]
fn deactivate_product_item_works() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		// Create item
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		// Deactivate
		assert_ok!(ProductItems::deactivate_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			0,
		));

		// Check inactive
		let item = crate::ProductItems::<Test>::get(&business_id, 0).unwrap();
		assert_eq!(item.is_active, false);

		// Check event
		System::assert_last_event(
			Event::ProductItemDeactivated {
				business_id,
				item_id: 0,
			}
			.into(),
		);
	});
}

#[test]
fn deactivate_already_inactive_fails() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		// Create and deactivate item
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		assert_ok!(ProductItems::deactivate_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			0,
		));

		// Try to deactivate again
		assert_noop!(
			ProductItems::deactivate_product_item(
				RuntimeOrigin::signed(caller.clone()),
				business_id,
				0,
			),
			Error::<Test>::ItemAlreadyInactive
		);
	});
}

#[test]
fn multiple_items_per_business_works() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_id = sample_business_id();

		// Create multiple items
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Volume".to_vec(),
			b"liter".to_vec(),
			None,
		));

		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_id,
			b"Length".to_vec(),
			b"meter".to_vec(),
			None,
		));

		// Check all items exist
		assert!(crate::ProductItems::<Test>::contains_key(&business_id, 0));
		assert!(crate::ProductItems::<Test>::contains_key(&business_id, 1));
		assert!(crate::ProductItems::<Test>::contains_key(&business_id, 2));

		// Check next ID counter
		assert_eq!(crate::NextItemId::<Test>::get(&business_id), 3);
	});
}

#[test]
fn different_businesses_can_have_same_item_names() {
	new_test_ext().execute_with(|| {
		let caller = AccountId32::from([1u8; 32]);
		let business_1 = [1u8; 16];
		let business_2 = [2u8; 16];

		// Create "Weight" item for business 1
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_1,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		// Create "Weight" item for business 2 (should work - different business)
		assert_ok!(ProductItems::create_product_item(
			RuntimeOrigin::signed(caller.clone()),
			business_2,
			b"Weight".to_vec(),
			b"kg".to_vec(),
			None,
		));

		// Both items exist
		assert!(crate::ProductItems::<Test>::contains_key(&business_1, 0));
		assert!(crate::ProductItems::<Test>::contains_key(&business_2, 0));
	});
}
