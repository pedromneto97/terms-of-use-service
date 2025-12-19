use domain::data::StorageServiceWithHealthCheck;
use google_cloud_storage::client::{Storage, StorageControl};
use tracing::info;

mod health_check;
mod service;

#[derive(Clone, Debug)]
pub struct GoogleCloudStorage {
    bucket: String,
    bucket_name: String,
    client: Storage,
    control_client: StorageControl,
}

#[inline]
async fn initialize_client() -> Storage {
    let client = Storage::builder().build().await;
    if let Err(err) = client {
        panic!("Failed to build Google Cloud Storage client: {err}");
    }

    client.unwrap()
}

#[inline]
async fn initialize_control_client() -> StorageControl {
    let control_client = StorageControl::builder().build().await;
    if let Err(err) = control_client {
        panic!("Failed to build Google Cloud Storage control client: {err}");
    }

    control_client.unwrap()
}

impl GoogleCloudStorage {
    /// Creates a new GoogleCloudStorage instance.
    ///
    /// This can be initialized in two ways:
    /// 1. Using Application Default Credentials (ADC) - set `GOOGLE_APPLICATION_CREDENTIALS` env var
    /// 2. Using explicit credentials from a JSON key file
    pub async fn new() -> Self {
        let bucket_name = std::env::var("GOOGLE_CLOUD_BUCKET")
            .expect("GOOGLE_CLOUD_BUCKET environment variable must be set");

        info!("Initializing Google Cloud Storage with bucket: {bucket_name}");

        let client = initialize_client().await;

        let control_client = initialize_control_client().await;

        GoogleCloudStorage {
            bucket: format!("projects/_/buckets/{bucket_name}"),
            bucket_name,
            client,
            control_client,
        }
    }
}

impl StorageServiceWithHealthCheck for GoogleCloudStorage {}
