use std::path::Path;

use domain::{
    data::service::StorageService,
    errors::{Result, TermsOfUseError},
};
use google_cloud_storage::client::{Storage, StorageControl};
use tokio::fs;
use tracing::{error, info};

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

impl StorageService for GoogleCloudStorage {
    async fn upload_file(&self, path: &Path, content_type: &str) -> Result<String> {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let file_extension =
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or(match content_type {
                    "application/pdf" => "pdf",
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    _ => "",
                });

        let object_name = format!("{}.{file_extension}", uuid::Uuid::new_v4());

        let file = fs::File::open(path).await.map_err(|err| {
            error!("Failed to open file for upload: {file_name} ({err})");

            TermsOfUseError::InternalServerError
        })?;

        self.client
            .write_object(&self.bucket, &object_name, file)
            .send_buffered()
            .await
            .map_err(|err| {
                error!("Failed to upload file to GCS: {file_name} -> {object_name} ({err})");

                TermsOfUseError::InternalServerError
            })?;

        info!(
            "Successfully uploaded file {file_name} to GCS: {object_name} in bucket {}",
            self.bucket_name
        );

        Ok(object_name)
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        self.control_client
            .delete_object()
            .set_bucket(&self.bucket)
            .set_object(path)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to delete file from GCS: {path} ({err})");

                TermsOfUseError::InternalServerError
            })?;

        info!("Successfully deleted file from GCS: {path}");

        Ok(())
    }

    async fn get_file_url(&self, path: &str) -> Result<String> {
        Ok(format!(
            "https://storage.googleapis.com/{}/{path}",
            self.bucket_name
        ))
    }
}
