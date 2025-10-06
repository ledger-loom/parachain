//! Unit tests for supply chain tracking pallet

use crate::{mock::*, Error, Event, EventType, TrackingStatus};
use frame::deps::sp_runtime::AccountId32;
use frame_support::{assert_noop, assert_ok};

type SupplyChainTracking = crate::Pallet<Test>;

#[test]
fn create_tracking_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;
		let location = b"Factory A, Beijing".to_vec();

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			location.clone()
		));

		// Verify tracking record exists
		assert!(crate::TrackingRecords::<Test>::contains_key(product_id));

		// Verify tracking details
		let record = crate::TrackingRecords::<Test>::get(product_id).unwrap();
		assert_eq!(record.product_id, product_id);
		assert_eq!(record.current_status, TrackingStatus::Created);
		assert_eq!(record.current_location.to_vec(), location);
		assert_eq!(record.events.len(), 0);

		// Verify product tracking mapping
		assert!(crate::ProductTracking::<Test>::contains_key(company_id, product_id));

		// Verify location index
		let location_hash = SupplyChainTracking::hash_location(&location);
		assert!(crate::LocationProducts::<Test>::contains_key(location_hash, product_id));

		// Verify event
		System::assert_last_event(
			Event::TrackingCreated {
				product_id,
				location,
			}
			.into(),
		);
	});
}

#[test]
fn create_tracking_fails_with_long_location() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;
		let location = vec![0u8; 200]; // Exceeds MaxLocationLength (128)

		assert_noop!(
			SupplyChainTracking::create_tracking(
				RuntimeOrigin::signed(user),
				product_id,
				company_id,
				location
			),
			Error::<Test>::LocationTooLong
		);
	});
}

#[test]
fn create_tracking_fails_for_duplicate_product() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;
		let location = b"Factory A".to_vec();

		// Create first tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			location.clone()
		));

		// Try to create duplicate
		assert_noop!(
			SupplyChainTracking::create_tracking(
				RuntimeOrigin::signed(user),
				product_id,
				company_id,
				location
			),
			Error::<Test>::TrackingAlreadyExists
		);
	});
}

#[test]
fn add_event_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;
		let initial_location = b"Factory A".to_vec();

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			initial_location.clone()
		));

		let new_location = b"Warehouse B".to_vec();
		let notes = b"Quality check passed".to_vec();

		// Add event
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::QualityCheck,
			new_location.clone(),
			notes.clone()
		));

		// Verify event added
		let record = crate::TrackingRecords::<Test>::get(product_id).unwrap();
		assert_eq!(record.events.len(), 1);
		assert_eq!(record.events[0].event_type, EventType::QualityCheck);
		assert_eq!(record.events[0].location.to_vec(), new_location);
		assert_eq!(record.events[0].notes.to_vec(), notes);
		assert_eq!(record.events[0].recorder, user);

		// Verify location updated
		assert_eq!(record.current_location.to_vec(), new_location);

		// Verify location index updated
		let old_location_hash = SupplyChainTracking::hash_location(&initial_location);
		assert!(!crate::LocationProducts::<Test>::contains_key(old_location_hash, product_id));

		let new_location_hash = SupplyChainTracking::hash_location(&new_location);
		assert!(crate::LocationProducts::<Test>::contains_key(new_location_hash, product_id));

		// Verify event
		System::assert_last_event(
			Event::EventAdded {
				product_id,
				event_type: EventType::QualityCheck,
			}
			.into(),
		);
	});
}

#[test]
fn add_event_fails_for_nonexistent_tracking() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 999u32;

		assert_noop!(
			SupplyChainTracking::add_event(
				RuntimeOrigin::signed(user),
				product_id,
				EventType::Shipped,
				b"Location".to_vec(),
				b"Notes".to_vec()
			),
			Error::<Test>::RecordNotFound
		);
	});
}

#[test]
fn add_event_fails_with_long_location() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			b"Factory A".to_vec()
		));

		let long_location = vec![0u8; 200]; // Exceeds MaxLocationLength (128)

		assert_noop!(
			SupplyChainTracking::add_event(
				RuntimeOrigin::signed(user),
				product_id,
				EventType::Shipped,
				long_location,
				b"Notes".to_vec()
			),
			Error::<Test>::LocationTooLong
		);
	});
}

#[test]
fn add_event_fails_with_long_notes() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			b"Factory A".to_vec()
		));

		let long_notes = vec![0u8; 600]; // Exceeds MaxNotesLength (512)

		assert_noop!(
			SupplyChainTracking::add_event(
				RuntimeOrigin::signed(user),
				product_id,
				EventType::Shipped,
				b"Location".to_vec(),
				long_notes
			),
			Error::<Test>::NotesTooLong
		);
	});
}

#[test]
fn add_multiple_events_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			b"Factory A".to_vec()
		));

		// Add multiple events
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::Manufactured,
			b"Factory A".to_vec(),
			b"Manufacturing complete".to_vec()
		));

		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::QualityCheck,
			b"Factory A".to_vec(),
			b"QC passed".to_vec()
		));

		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::Shipped,
			b"Warehouse B".to_vec(),
			b"Shipped to warehouse".to_vec()
		));

		// Verify all events added
		let record = crate::TrackingRecords::<Test>::get(product_id).unwrap();
		assert_eq!(record.events.len(), 3);
		assert_eq!(record.events[0].event_type, EventType::Manufactured);
		assert_eq!(record.events[1].event_type, EventType::QualityCheck);
		assert_eq!(record.events[2].event_type, EventType::Shipped);
	});
}

#[test]
fn update_status_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			b"Factory A".to_vec()
		));

		// Update status
		assert_ok!(SupplyChainTracking::update_status(
			RuntimeOrigin::signed(user),
			product_id,
			TrackingStatus::InProgress
		));

		// Verify status change
		let record = crate::TrackingRecords::<Test>::get(product_id).unwrap();
		assert_eq!(record.current_status, TrackingStatus::InProgress);

		// Verify event
		System::assert_last_event(
			Event::StatusUpdated {
				product_id,
				status: TrackingStatus::InProgress,
			}
			.into(),
		);
	});
}

#[test]
fn update_status_fails_for_nonexistent_tracking() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 999u32;

		assert_noop!(
			SupplyChainTracking::update_status(
				RuntimeOrigin::signed(user),
				product_id,
				TrackingStatus::Completed
			),
			Error::<Test>::RecordNotFound
		);
	});
}

#[test]
fn update_location_works() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;
		let initial_location = b"Factory A".to_vec();

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			initial_location.clone()
		));

		let new_location = b"Distribution Center C".to_vec();

		// Update location
		assert_ok!(SupplyChainTracking::update_location(
			RuntimeOrigin::signed(user),
			product_id,
			new_location.clone()
		));

		// Verify location change
		let record = crate::TrackingRecords::<Test>::get(product_id).unwrap();
		assert_eq!(record.current_location.to_vec(), new_location);

		// Verify location index updated
		let old_location_hash = SupplyChainTracking::hash_location(&initial_location);
		assert!(!crate::LocationProducts::<Test>::contains_key(old_location_hash, product_id));

		let new_location_hash = SupplyChainTracking::hash_location(&new_location);
		assert!(crate::LocationProducts::<Test>::contains_key(new_location_hash, product_id));

		// Verify event
		System::assert_last_event(
			Event::LocationUpdated {
				product_id,
				new_location,
			}
			.into(),
		);
	});
}

#[test]
fn update_location_fails_for_nonexistent_tracking() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 999u32;

		assert_noop!(
			SupplyChainTracking::update_location(
				RuntimeOrigin::signed(user),
				product_id,
				b"New Location".to_vec()
			),
			Error::<Test>::RecordNotFound
		);
	});
}

#[test]
fn update_location_fails_with_long_location() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;

		// Create tracking record
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			b"Factory A".to_vec()
		));

		let long_location = vec![0u8; 200]; // Exceeds MaxLocationLength (128)

		assert_noop!(
			SupplyChainTracking::update_location(
				RuntimeOrigin::signed(user),
				product_id,
				long_location
			),
			Error::<Test>::LocationTooLong
		);
	});
}

#[test]
fn complete_tracking_workflow() {
	new_test_ext().execute_with(|| {
		let user = AccountId32::from([1u8; 32]);
		let product_id = 1u32;
		let company_id = 1u32;

		// Step 1: Create tracking
		assert_ok!(SupplyChainTracking::create_tracking(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			company_id,
			b"Factory A, Beijing".to_vec()
		));

		// Step 2: Manufactured
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::Manufactured,
			b"Factory A, Beijing".to_vec(),
			b"Product manufactured successfully".to_vec()
		));
		assert_ok!(SupplyChainTracking::update_status(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			TrackingStatus::InProgress
		));

		// Step 3: Quality check
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::QualityCheck,
			b"Factory A, Beijing".to_vec(),
			b"Quality inspection passed".to_vec()
		));

		// Step 4: Shipped
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::Shipped,
			b"In Transit to Warehouse B".to_vec(),
			b"Shipped via FedEx tracking #12345".to_vec()
		));

		// Step 5: In transit
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::InTransit,
			b"Shanghai Port".to_vec(),
			b"Arrived at port for customs clearance".to_vec()
		));

		// Step 6: Delivered
		assert_ok!(SupplyChainTracking::add_event(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			EventType::Delivered,
			b"Warehouse B, Los Angeles".to_vec(),
			b"Product delivered to warehouse".to_vec()
		));
		assert_ok!(SupplyChainTracking::update_status(
			RuntimeOrigin::signed(user.clone()),
			product_id,
			TrackingStatus::Completed
		));

		// Verify final state
		let record = crate::TrackingRecords::<Test>::get(product_id).unwrap();
		assert_eq!(record.current_status, TrackingStatus::Completed);
		assert_eq!(record.current_location.to_vec(), b"Warehouse B, Los Angeles".to_vec());
		assert_eq!(record.events.len(), 5);

		// Verify location index points to final location
		let final_location_hash = SupplyChainTracking::hash_location(b"Warehouse B, Los Angeles");
		assert!(crate::LocationProducts::<Test>::contains_key(final_location_hash, product_id));
	});
}
