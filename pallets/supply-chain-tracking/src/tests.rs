//! Unit tests for supply chain tracking pallet (append-only architecture)

use crate::{mock::*, Error, Event};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

type SupplyChainTracking = crate::Pallet<Test>;

#[test]
fn create_item_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;
		let status = b"Created".to_vec();
		let location = Some(b"Factory A, Beijing".to_vec());
		let encrypted_data = b"encrypted_initial_data".to_vec();

		// Create item with first record
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			status.clone(),
			location.clone(),
			encrypted_data.clone()
		));

		// Verify item metadata exists
		assert!(crate::Items::<Test>::contains_key(&item_id));
		let metadata = crate::Items::<Test>::get(&item_id).unwrap();
		assert_eq!(metadata.business_id, business_id);
		assert_eq!(metadata.record_count, 1);
		assert_eq!(metadata.creator, creator);

		// Verify first record exists
		assert!(crate::ItemRecords::<Test>::contains_key(&item_id, 0));
		let record = crate::ItemRecords::<Test>::get(&item_id, 0).unwrap();
		assert_eq!(record.item_id, item_id);
		assert_eq!(record.business_id, business_id);
		assert_eq!(record.sequence, 0);
		assert_eq!(record.status.to_vec(), status);
		assert_eq!(record.location.as_ref().map(|l| l.to_vec()), location);
		assert_eq!(record.recorder, creator);
		assert_eq!(record.previous_hash, [0u8; 32]); // Genesis record has zero hash

		// Verify chain items mapping
		assert!(crate::BusinessItems::<Test>::contains_key(business_id, &item_id));

		// Verify event
		System::assert_last_event(
			Event::ItemCreated {
				item_id,
				business_id,
				creator,
			}
			.into(),
		);
	});
}

#[test]
fn create_item_fails_if_already_exists() {
	new_test_ext().execute_with(|| {
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Create item
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			b"Created".to_vec(),
			None,
			b"data".to_vec()
		));

		// Try to create duplicate
		assert_noop!(
			SupplyChainTracking::create_item(
				RuntimeOrigin::signed(creator),
				item_id,
				business_id,
				b"Created".to_vec(),
				None,
				b"data".to_vec()
			),
			Error::<Test>::ItemAlreadyExists
		);
	});
}

#[test]
fn create_item_fails_with_long_status() {
	new_test_ext().execute_with(|| {
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let long_status = vec![0u8; 100]; // Exceeds MaxStatusLength (64)

		assert_noop!(
			SupplyChainTracking::create_item(
				RuntimeOrigin::signed(creator),
				item_id,
				1u32,
				long_status,
				None,
				b"data".to_vec()
			),
			Error::<Test>::StatusTooLong
		);
	});
}

#[test]
fn create_item_fails_with_long_location() {
	new_test_ext().execute_with(|| {
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let long_location = vec![0u8; 200]; // Exceeds MaxLocationLength (128)

		assert_noop!(
			SupplyChainTracking::create_item(
				RuntimeOrigin::signed(creator),
				item_id,
				1u32,
				b"Created".to_vec(),
				Some(long_location),
				b"data".to_vec()
			),
			Error::<Test>::LocationTooLong
		);
	});
}

#[test]
fn create_item_fails_with_long_data() {
	new_test_ext().execute_with(|| {
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let long_data = vec![0u8; 2000]; // Exceeds MaxEncryptedDataLength (1024)

		assert_noop!(
			SupplyChainTracking::create_item(
				RuntimeOrigin::signed(creator),
				item_id,
				1u32,
				b"Created".to_vec(),
				None,
				long_data
			),
			Error::<Test>::EncryptedDataTooLong
		);
	});
}

#[test]
fn append_record_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let creator = AccountId32::from([1u8; 32]);
		let recorder = AccountId32::from([2u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Create item
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			b"Created".to_vec(),
			Some(b"Factory A".to_vec()),
			b"initial_data".to_vec()
		));

		// Get first record hash
		let record0 = crate::ItemRecords::<Test>::get(&item_id, 0).unwrap();
		let record0_hash = record0.record_hash;

		System::set_block_number(2);

		// Append second record
		assert_ok!(SupplyChainTracking::append_record(
			RuntimeOrigin::signed(recorder.clone()),
			item_id,
			b"Manufactured".to_vec(),
			Some(b"Factory A".to_vec()),
			b"second_record_data".to_vec()
		));

		// Verify metadata updated
		let metadata = crate::Items::<Test>::get(&item_id).unwrap();
		assert_eq!(metadata.record_count, 2);
		assert_eq!(metadata.latest_hash, crate::ItemRecords::<Test>::get(&item_id, 1).unwrap().record_hash);

		// Verify second record
		let record1 = crate::ItemRecords::<Test>::get(&item_id, 1).unwrap();
		assert_eq!(record1.sequence, 1);
		assert_eq!(record1.status.to_vec(), b"Manufactured");
		assert_eq!(record1.recorder, recorder);
		assert_eq!(record1.previous_hash, record0_hash); // Links to first record

		// Verify event
		System::assert_last_event(
			Event::RecordAppended {
				item_id,
				sequence: 1,
				recorder,
			}
			.into(),
		);
	});
}

#[test]
fn append_record_fails_for_nonexistent_item() {
	new_test_ext().execute_with(|| {
		let recorder = AccountId32::from([1u8; 32]);
		let item_id = [99u8; 32];

		assert_noop!(
			SupplyChainTracking::append_record(
				RuntimeOrigin::signed(recorder),
				item_id,
				b"Status".to_vec(),
				None,
				b"data".to_vec()
			),
			Error::<Test>::ItemNotFound
		);
	});
}

#[test]
fn query_item_history_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Create item
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			b"Created".to_vec(),
			None,
			b"data0".to_vec()
		));

		// Append multiple records
		for i in 1..=3 {
			System::set_block_number(i + 1);
			assert_ok!(SupplyChainTracking::append_record(
				RuntimeOrigin::signed(creator.clone()),
				item_id,
				format!("Status{}", i).as_bytes().to_vec(),
				None,
				format!("data{}", i).as_bytes().to_vec()
			));
		}

		// Query history
		assert_ok!(SupplyChainTracking::query_item_history(
			RuntimeOrigin::signed(creator),
			item_id
		));

		// Verify event with all records
		let expected_event = Event::ItemHistoryQueried {
			item_id,
			record_count: 4,
		};
		System::assert_last_event(expected_event.into());
	});
}

#[test]
fn verify_item_chain_works_for_valid_chain() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Create item with multiple records
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			b"Created".to_vec(),
			None,
			b"data0".to_vec()
		));

		for i in 1..=5 {
			System::set_block_number(i + 1);
			assert_ok!(SupplyChainTracking::append_record(
				RuntimeOrigin::signed(creator.clone()),
				item_id,
				format!("Status{}", i).as_bytes().to_vec(),
				None,
				format!("data{}", i).as_bytes().to_vec()
			));
		}

		// Verify chain integrity
		assert_ok!(SupplyChainTracking::verify_item_chain(
			RuntimeOrigin::signed(creator),
			item_id
		));

		// Verify event
		System::assert_last_event(
			Event::ItemChainVerified {
				item_id,
				is_valid: true,
			}
			.into(),
		);
	});
}

#[test]
fn complete_item_lifecycle() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let manufacturer = AccountId32::from([1u8; 32]);
		let quality_checker = AccountId32::from([2u8; 32]);
		let shipper = AccountId32::from([3u8; 32]);
		let warehouse = AccountId32::from([4u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Step 1: Item created at factory
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(manufacturer.clone()),
			item_id,
			business_id,
			b"Raw Material".to_vec(),
			Some(b"Factory A, Beijing".to_vec()),
			b"encrypted_raw_material_data".to_vec()
		));

		// Step 2: Manufacturing
		System::set_block_number(2);
		assert_ok!(SupplyChainTracking::append_record(
			RuntimeOrigin::signed(manufacturer.clone()),
			item_id,
			b"Manufactured".to_vec(),
			Some(b"Factory A, Beijing".to_vec()),
			b"encrypted_manufacturing_data".to_vec()
		));

		// Step 3: Quality check
		System::set_block_number(3);
		assert_ok!(SupplyChainTracking::append_record(
			RuntimeOrigin::signed(quality_checker.clone()),
			item_id,
			b"QC Passed".to_vec(),
			Some(b"QC Department, Factory A".to_vec()),
			b"encrypted_qc_data".to_vec()
		));

		// Step 4: Packaging and ready to ship
		System::set_block_number(4);
		assert_ok!(SupplyChainTracking::append_record(
			RuntimeOrigin::signed(manufacturer.clone()),
			item_id,
			b"Packaged".to_vec(),
			Some(b"Packaging Dept, Factory A".to_vec()),
			b"encrypted_packaging_data".to_vec()
		));

		// Step 5: Shipped
		System::set_block_number(5);
		assert_ok!(SupplyChainTracking::append_record(
			RuntimeOrigin::signed(shipper.clone()),
			item_id,
			b"In Transit".to_vec(),
			Some(b"En route to Warehouse B".to_vec()),
			b"encrypted_shipping_data".to_vec()
		));

		// Step 6: Arrived at warehouse
		System::set_block_number(6);
		assert_ok!(SupplyChainTracking::append_record(
			RuntimeOrigin::signed(warehouse.clone()),
			item_id,
			b"Delivered".to_vec(),
			Some(b"Warehouse B, Los Angeles".to_vec()),
			b"encrypted_delivery_data".to_vec()
		));

		// Verify complete chain
		let metadata = crate::Items::<Test>::get(&item_id).unwrap();
		assert_eq!(metadata.record_count, 6);
		assert_eq!(metadata.creator, manufacturer);

		// Verify all records exist and are linked
		for seq in 0..6 {
			assert!(crate::ItemRecords::<Test>::contains_key(&item_id, seq));
		}

		// Verify hash chain integrity
		for seq in 1..6 {
			let current_record = crate::ItemRecords::<Test>::get(&item_id, seq).unwrap();
			let previous_record = crate::ItemRecords::<Test>::get(&item_id, seq - 1).unwrap();
			assert_eq!(current_record.previous_hash, previous_record.record_hash);
		}

		// Verify chain integrity check
		assert_ok!(SupplyChainTracking::verify_item_chain(
			RuntimeOrigin::signed(warehouse),
			item_id
		));
	});
}

#[test]
fn get_item_history_helper_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Create item
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			b"Created".to_vec(),
			None,
			b"data".to_vec()
		));

		// Add more records
		for i in 1..=3 {
			System::set_block_number(i + 1);
			assert_ok!(SupplyChainTracking::append_record(
				RuntimeOrigin::signed(creator.clone()),
				item_id,
				format!("Status{}", i).as_bytes().to_vec(),
				None,
				b"data".to_vec()
			));
		}

		// Use helper function
		let history = SupplyChainTracking::get_item_history(item_id).unwrap();
		assert_eq!(history.len(), 4);

		// Verify sequence order
		for (i, record) in history.iter().enumerate() {
			assert_eq!(record.sequence, i as u32);
		}
	});
}

#[test]
fn hash_computation_is_deterministic() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let creator = AccountId32::from([1u8; 32]);
		let item_id = [1u8; 32];
		let business_id = 1u32;

		// Create two identical items with different IDs
		let item_id_2 = [2u8; 32];

		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id,
			business_id,
			b"Created".to_vec(),
			None,
			b"same_data".to_vec()
		));

		let record1 = crate::ItemRecords::<Test>::get(&item_id, 0).unwrap();
		let hash1 = record1.record_hash;

		// Create second item (hash should be different because item_id is different)
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator.clone()),
			item_id_2,
			business_id,
			b"Created".to_vec(),
			None,
			b"same_data".to_vec()
		));

		let record2 = crate::ItemRecords::<Test>::get(&item_id_2, 0).unwrap();
		let hash2 = record2.record_hash;

		// Hashes should be different (different item IDs)
		assert_ne!(hash1, hash2);
	});
}

#[test]
fn multiple_chains_work_independently() {
	new_test_ext().execute_with(|| {
		let creator1 = AccountId32::from([1u8; 32]);
		let creator2 = AccountId32::from([2u8; 32]);
		let item_id_1 = [1u8; 32];
		let item_id_2 = [2u8; 32];
		let business_id_1 = 1u32;
		let business_id_2 = 2u32;

		// Create items for different chains
		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator1.clone()),
			item_id_1,
			business_id_1,
			b"Created".to_vec(),
			None,
			b"data1".to_vec()
		));

		assert_ok!(SupplyChainTracking::create_item(
			RuntimeOrigin::signed(creator2.clone()),
			item_id_2,
			business_id_2,
			b"Created".to_vec(),
			None,
			b"data2".to_vec()
		));

		// Verify chain separation
		assert!(crate::BusinessItems::<Test>::contains_key(business_id_1, &item_id_1));
		assert!(!crate::BusinessItems::<Test>::contains_key(business_id_1, &item_id_2));

		assert!(crate::BusinessItems::<Test>::contains_key(business_id_2, &item_id_2));
		assert!(!crate::BusinessItems::<Test>::contains_key(business_id_2, &item_id_1));
	});
}
