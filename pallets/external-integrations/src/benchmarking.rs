//! Benchmarking setup for pallet-external-integrations

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_api_key() {
		let caller: T::AccountId = whitelisted_caller();
		let name = b"Test API Key".to_vec();
		let permissions = vec![ApiPermission::ReadProducts, ApiPermission::WriteProducts];

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), name, permissions, 100);
	}

	#[benchmark]
	fn revoke_api_key() {
		let caller: T::AccountId = whitelisted_caller();
		let name = b"Test API Key".to_vec();
		let permissions = vec![ApiPermission::ReadProducts];

		// Setup: Create API key first
		Pallet::<T>::create_api_key(
			RawOrigin::Signed(caller.clone()).into(),
			name,
			permissions,
			100,
		)
		.unwrap();

		let key_hash = T::Hashing::hash_of(&(caller.clone(), b"Test API Key".to_vec(), 0u32));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), key_hash);
	}

	#[benchmark]
	fn create_import_job() {
		let caller: T::AccountId = whitelisted_caller();
		let data_hash = T::Hashing::hash_of(&b"import_data".to_vec());

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), ImportType::Products, data_hash, 100);
	}

	#[benchmark]
	fn create_export_job() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), ExportType::Products, ExportFormat::CSV);
	}

	#[benchmark]
	fn queue_email() {
		let caller: T::AccountId = whitelisted_caller();

		// Setup: Create email template first
		Pallet::<T>::update_email_template(
			RawOrigin::Signed(caller.clone()).into(),
			EmailTemplateType::ProductUpdate,
			b"Template".to_vec(),
			b"Subject".to_vec(),
			b"Body".to_vec(),
		)
		.unwrap();

		let recipients = vec![b"test@example.com".to_vec()];
		let variables = vec![(b"key".to_vec(), b"value".to_vec())];

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			recipients,
			EmailTemplateType::ProductUpdate,
			variables
		);
	}

	#[benchmark]
	fn update_email_template() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			EmailTemplateType::ProductUpdate,
			b"Template Name".to_vec(),
			b"Email Subject".to_vec(),
			b"Email Body".to_vec()
		);
	}

	#[benchmark]
	fn register_barcode() {
		let caller: T::AccountId = whitelisted_caller();
		let entity_id = T::Hashing::hash_of(&b"product_123".to_vec());

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			b"1234567890123".to_vec(),
			BarcodeEntityType::Product,
			entity_id,
			BarcodeType::EAN13
		);
	}

	#[benchmark]
	fn scan_barcode() {
		let caller: T::AccountId = whitelisted_caller();
		let scanner: T::AccountId = whitelisted_caller();
		let entity_id = T::Hashing::hash_of(&b"product_123".to_vec());
		let barcode = b"1234567890123".to_vec();

		// Setup: Register barcode first
		Pallet::<T>::register_barcode(
			RawOrigin::Signed(caller).into(),
			barcode.clone(),
			BarcodeEntityType::Product,
			entity_id,
			BarcodeType::EAN13,
		)
		.unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(scanner), barcode);
	}

	#[benchmark]
	fn register_webhook() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			b"https://example.com/webhook".to_vec(),
			vec![WebhookEventType::ProductCreated],
			Some(b"secret".to_vec())
		);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
