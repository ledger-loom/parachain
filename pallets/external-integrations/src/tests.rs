use crate::{mock::*, Error, Event, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_api_key_works() {
	new_test_ext().execute_with(|| {
		// Create API key
		assert_ok!(ExternalIntegrations::create_api_key(
			RuntimeOrigin::signed(1),
			b"My API Key".to_vec(),
			vec![ApiPermission::ReadProducts, ApiPermission::WriteProducts],
			100, // rate limit
		));

		// Check event was emitted
		System::assert_last_event(
			Event::ApiKeyCreated {
				account: 1,
				key_hash: sp_core::H256::from_low_u64_be(0), // Mock hash
				name: b"My API Key".to_vec(),
			}
			.into(),
		);
	});
}

#[test]
fn create_api_key_fails_when_max_reached() {
	new_test_ext().execute_with(|| {
		// Create max API keys
		for i in 0..10 {
			assert_ok!(ExternalIntegrations::create_api_key(
				RuntimeOrigin::signed(1),
				format!("Key {}", i).into_bytes(),
				vec![ApiPermission::ReadProducts],
				100,
			));
		}

		// Try to create one more
		assert_noop!(
			ExternalIntegrations::create_api_key(
				RuntimeOrigin::signed(1),
				b"Extra Key".to_vec(),
				vec![ApiPermission::ReadProducts],
				100,
			),
			Error::<Test>::MaxApiKeysReached
		);
	});
}

#[test]
fn revoke_api_key_works() {
	new_test_ext().execute_with(|| {
		// Create API key
		assert_ok!(ExternalIntegrations::create_api_key(
			RuntimeOrigin::signed(1),
			b"My API Key".to_vec(),
			vec![ApiPermission::ReadProducts],
			100,
		));

		let key_hash = sp_core::H256::from_low_u64_be(0); // Mock hash

		// Revoke API key
		assert_ok!(ExternalIntegrations::revoke_api_key(
			RuntimeOrigin::signed(1),
			key_hash,
		));

		// Check event was emitted
		System::assert_last_event(
			Event::ApiKeyRevoked {
				account: 1,
				key_hash,
			}
			.into(),
		);
	});
}

#[test]
fn create_import_job_works() {
	new_test_ext().execute_with(|| {
		let data_hash = sp_core::H256::from_low_u64_be(123);

		assert_ok!(ExternalIntegrations::create_import_job(
			RuntimeOrigin::signed(1),
			ImportType::Products,
			data_hash,
			50, // item count
		));

		// Check event was emitted
		let job_id = sp_core::H256::from_low_u64_be(0); // Mock job ID
		System::assert_last_event(
			Event::ImportJobCreated {
				job_id,
				account: 1,
				item_count: 50,
			}
			.into(),
		);
	});
}

#[test]
fn create_import_job_fails_with_large_batch() {
	new_test_ext().execute_with(|| {
		let data_hash = sp_core::H256::from_low_u64_be(123);

		// Try to create import job with too many items
		assert_noop!(
			ExternalIntegrations::create_import_job(
				RuntimeOrigin::signed(1),
				ImportType::Products,
				data_hash,
				2000, // exceeds MaxBatchSize
			),
			Error::<Test>::BatchSizeExceeded
		);
	});
}

#[test]
fn create_export_job_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExternalIntegrations::create_export_job(
			RuntimeOrigin::signed(1),
			ExportType::Products,
			ExportFormat::CSV,
		));

		// Check event was emitted
		let job_id = sp_core::H256::from_low_u64_be(0); // Mock job ID
		System::assert_last_event(
			Event::ExportJobCreated {
				job_id,
				account: 1,
				export_type: ExportType::Products,
			}
			.into(),
		);
	});
}

#[test]
fn queue_email_works() {
	new_test_ext().execute_with(|| {
		// First create an email template
		assert_ok!(ExternalIntegrations::update_email_template(
			RuntimeOrigin::signed(1),
			EmailTemplateType::ProductUpdate,
			b"Product Update".to_vec(),
			b"Your product has been updated".to_vec(),
			b"Product {{product_name}} has been updated.".to_vec(),
		));

		// Queue email
		assert_ok!(ExternalIntegrations::queue_email(
			RuntimeOrigin::signed(1),
			vec![b"user@example.com".to_vec()],
			EmailTemplateType::ProductUpdate,
			vec![(b"product_name".to_vec(), b"Coffee Beans".to_vec())],
		));

		// Check event was emitted
		let email_id = sp_core::H256::from_low_u64_be(0); // Mock email ID
		System::assert_last_event(
			Event::EmailQueued {
				email_id,
				recipient: b"user@example.com".to_vec(),
				template_type: EmailTemplateType::ProductUpdate,
			}
			.into(),
		);
	});
}

#[test]
fn queue_email_fails_without_template() {
	new_test_ext().execute_with(|| {
		// Try to queue email without creating template first
		assert_noop!(
			ExternalIntegrations::queue_email(
				RuntimeOrigin::signed(1),
				vec![b"user@example.com".to_vec()],
				EmailTemplateType::ProductUpdate,
				vec![],
			),
			Error::<Test>::EmailTemplateNotFound
		);
	});
}

#[test]
fn register_barcode_works() {
	new_test_ext().execute_with(|| {
		let entity_id = sp_core::H256::from_low_u64_be(456);

		assert_ok!(ExternalIntegrations::register_barcode(
			RuntimeOrigin::signed(1),
			b"12345678901234".to_vec(),
			BarcodeEntityType::Product,
			entity_id,
			BarcodeType::EAN13,
		));

		// Check event was emitted
		System::assert_last_event(
			Event::BarcodeRegistered {
				barcode: b"12345678901234".to_vec(),
				entity_type: BarcodeEntityType::Product,
				entity_id,
			}
			.into(),
		);
	});
}

#[test]
fn register_barcode_fails_if_already_exists() {
	new_test_ext().execute_with(|| {
		let entity_id = sp_core::H256::from_low_u64_be(456);
		let barcode = b"12345678901234".to_vec();

		// Register barcode
		assert_ok!(ExternalIntegrations::register_barcode(
			RuntimeOrigin::signed(1),
			barcode.clone(),
			BarcodeEntityType::Product,
			entity_id,
			BarcodeType::EAN13,
		));

		// Try to register same barcode again
		assert_noop!(
			ExternalIntegrations::register_barcode(
				RuntimeOrigin::signed(1),
				barcode,
				BarcodeEntityType::Product,
				entity_id,
				BarcodeType::EAN13,
			),
			Error::<Test>::BarcodeAlreadyRegistered
		);
	});
}

#[test]
fn scan_barcode_works() {
	new_test_ext().execute_with(|| {
		let entity_id = sp_core::H256::from_low_u64_be(456);
		let barcode = b"12345678901234".to_vec();

		// Register barcode first
		assert_ok!(ExternalIntegrations::register_barcode(
			RuntimeOrigin::signed(1),
			barcode.clone(),
			BarcodeEntityType::Product,
			entity_id,
			BarcodeType::EAN13,
		));

		// Scan barcode
		assert_ok!(ExternalIntegrations::scan_barcode(
			RuntimeOrigin::signed(2),
			barcode.clone(),
		));

		// Check event was emitted
		System::assert_last_event(
			Event::BarcodeScanned {
				barcode,
				scanner: 2,
				timestamp: 0,
			}
			.into(),
		);
	});
}

#[test]
fn scan_barcode_fails_if_not_registered() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			ExternalIntegrations::scan_barcode(
				RuntimeOrigin::signed(1),
				b"99999999999999".to_vec(),
			),
			Error::<Test>::BarcodeNotFound
		);
	});
}

#[test]
fn register_webhook_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExternalIntegrations::register_webhook(
			RuntimeOrigin::signed(1),
			b"https://example.com/webhook".to_vec(),
			vec![WebhookEventType::ProductCreated, WebhookEventType::ProductUpdated],
			Some(b"secret123".to_vec()),
		));

		// Check event was emitted
		System::assert_last_event(
			Event::WebhookRegistered {
				account: 1,
				url: b"https://example.com/webhook".to_vec(),
				event_types: vec![WebhookEventType::ProductCreated, WebhookEventType::ProductUpdated],
			}
			.into(),
		);
	});
}

#[test]
fn update_email_template_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExternalIntegrations::update_email_template(
			RuntimeOrigin::signed(1),
			EmailTemplateType::UserInvitation,
			b"User Invitation".to_vec(),
			b"You're invited!".to_vec(),
			b"Hello {{user_name}}, you've been invited to join {{company_name}}.".to_vec(),
		));

		// Check event was emitted
		System::assert_last_event(
			Event::EmailTemplateUpdated {
				template_type: EmailTemplateType::UserInvitation,
				name: b"User Invitation".to_vec(),
			}
			.into(),
		);
	});
}
