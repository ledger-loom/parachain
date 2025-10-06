//! Unit tests for product management pallet

use crate::{mock::*, Error, Event, ProductStatus};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

type ProductManagement = crate::Pallet<Test>;

#[test]
fn create_product_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = b"Test Product".to_vec();
		let category = b"Electronics".to_vec();
		let attributes = vec![
			(b"color".to_vec(), b"blue".to_vec()),
			(b"size".to_vec(), b"large".to_vec()),
		];

		// Create product
		assert_ok!(ProductManagement::create_product(
			RuntimeOrigin::signed(user.clone()),
			company_id,
			name.clone(),
			category.clone(),
			attributes.clone()
		));

		// Verify product exists
		let product_id = 0u32;
		assert!(crate::Products::<Test>::contains_key(product_id));

		// Verify product details
		let product = crate::Products::<Test>::get(product_id).unwrap();
		assert_eq!(product.name.to_vec(), name);
		assert_eq!(product.category.to_vec(), category);
		assert_eq!(product.company_id, company_id);
		assert_eq!(product.status, ProductStatus::Active);
		assert_eq!(product.attributes.len(), 2);

		// Verify company products mapping
		assert!(crate::CompanyProducts::<Test>::contains_key(company_id, product_id));

		// Verify next product ID
		assert_eq!(crate::NextProductId::<Test>::get(), 1);

		// Verify category count
		let category_bounded = category.clone().try_into().unwrap();
		assert_eq!(crate::Categories::<Test>::get(&category_bounded), Some(1));

		// Verify event
		System::assert_last_event(
			Event::ProductCreated {
				product_id,
				company_id,
				name,
			}
			.into(),
		);
	});
}

#[test]
fn create_product_fails_with_long_name() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = vec![0u8; 300]; // Exceeds MaxProductNameLength (256)
		let category = b"Electronics".to_vec();

		assert_noop!(
			ProductManagement::create_product(
				RuntimeOrigin::signed(user),
				company_id,
				name,
				category,
				vec![]
			),
			Error::<Test>::ProductNameTooLong
		);
	});
}

#[test]
fn update_product_status_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = b"Test Product".to_vec();
		let category = b"Electronics".to_vec();

		// Create product
		assert_ok!(ProductManagement::create_product(
			RuntimeOrigin::signed(user.clone()),
			company_id,
			name,
			category,
			vec![]
		));

		let product_id = 0u32;

		// Update status
		assert_ok!(ProductManagement::update_product_status(
			RuntimeOrigin::signed(user),
			product_id,
			ProductStatus::Inactive
		));

		// Verify status change
		let product = crate::Products::<Test>::get(product_id).unwrap();
		assert_eq!(product.status, ProductStatus::Inactive);

		// Verify event
		System::assert_last_event(
			Event::ProductStatusChanged {
				product_id,
				status: ProductStatus::Inactive,
			}
			.into(),
		);
	});
}

#[test]
fn update_product_status_fails_for_nonexistent_product() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 999u32;

		assert_noop!(
			ProductManagement::update_product_status(
				RuntimeOrigin::signed(user),
				product_id,
				ProductStatus::Inactive
			),
			Error::<Test>::ProductNotFound
		);
	});
}

#[test]
fn add_attribute_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = b"Test Product".to_vec();
		let category = b"Electronics".to_vec();

		// Create product
		assert_ok!(ProductManagement::create_product(
			RuntimeOrigin::signed(user.clone()),
			company_id,
			name,
			category,
			vec![]
		));

		let product_id = 0u32;
		let key = b"warranty".to_vec();
		let value = b"2 years".to_vec();

		// Add attribute
		assert_ok!(ProductManagement::add_attribute(
			RuntimeOrigin::signed(user),
			product_id,
			key.clone(),
			value.clone()
		));

		// Verify attribute added
		let product = crate::Products::<Test>::get(product_id).unwrap();
		assert_eq!(product.attributes.len(), 1);
		assert_eq!(product.attributes[0].key.to_vec(), key);
		assert_eq!(product.attributes[0].value.to_vec(), value);

		// Verify event
		System::assert_last_event(
			Event::AttributeAdded {
				product_id,
				key,
			}
			.into(),
		);
	});
}

#[test]
fn add_attribute_fails_for_duplicate_key() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = b"Test Product".to_vec();
		let category = b"Electronics".to_vec();
		let key = b"color".to_vec();

		// Create product with initial attribute
		assert_ok!(ProductManagement::create_product(
			RuntimeOrigin::signed(user.clone()),
			company_id,
			name,
			category,
			vec![(key.clone(), b"blue".to_vec())]
		));

		let product_id = 0u32;

		// Try to add duplicate attribute
		assert_noop!(
			ProductManagement::add_attribute(
				RuntimeOrigin::signed(user),
				product_id,
				key,
				b"red".to_vec()
			),
			Error::<Test>::DuplicateAttribute
		);
	});
}

#[test]
fn add_attribute_fails_for_nonexistent_product() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 999u32;

		assert_noop!(
			ProductManagement::add_attribute(
				RuntimeOrigin::signed(user),
				product_id,
				b"key".to_vec(),
				b"value".to_vec()
			),
			Error::<Test>::ProductNotFound
		);
	});
}

#[test]
fn update_attribute_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = b"Test Product".to_vec();
		let category = b"Electronics".to_vec();
		let key = b"color".to_vec();

		// Create product with initial attribute
		assert_ok!(ProductManagement::create_product(
			RuntimeOrigin::signed(user.clone()),
			company_id,
			name,
			category,
			vec![(key.clone(), b"blue".to_vec())]
		));

		let product_id = 0u32;
		let new_value = b"red".to_vec();

		// Update attribute
		assert_ok!(ProductManagement::update_attribute(
			RuntimeOrigin::signed(user),
			product_id,
			key.clone(),
			new_value.clone()
		));

		// Verify attribute updated
		let product = crate::Products::<Test>::get(product_id).unwrap();
		assert_eq!(product.attributes[0].value.to_vec(), new_value);

		// Verify event
		System::assert_last_event(
			Event::AttributeUpdated {
				product_id,
				key,
			}
			.into(),
		);
	});
}

#[test]
fn update_attribute_fails_for_nonexistent_attribute() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let company_id = 1u32;
		let name = b"Test Product".to_vec();
		let category = b"Electronics".to_vec();

		// Create product without attributes
		assert_ok!(ProductManagement::create_product(
			RuntimeOrigin::signed(user.clone()),
			company_id,
			name,
			category,
			vec![]
		));

		let product_id = 0u32;

		// Try to update non-existent attribute
		assert_noop!(
			ProductManagement::update_attribute(
				RuntimeOrigin::signed(user),
				product_id,
				b"nonexistent".to_vec(),
				b"value".to_vec()
			),
			Error::<Test>::AttributeNotFound
		);
	});
}
