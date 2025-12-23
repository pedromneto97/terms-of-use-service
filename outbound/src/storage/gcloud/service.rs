use std::path::Path;

use async_trait::async_trait;
use domain::{
    data::service::StorageService,
    errors::{Result, TermsOfUseError},
};
use tokio::fs;
use tracing::{error, info};

use crate::GoogleCloudStorage;

#[async_trait]
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

#[cfg(test)]
mod tests {
    use domain::data::service::StorageService;
    use google_cloud_storage::client::{Storage, StorageControl};

    use super::GoogleCloudStorage;

    async fn build_test_storage(bucket_name: &str) -> GoogleCloudStorage {
        let client = Storage::builder()
            .build()
            .await
            .expect("Failed to build test Storage client");
        let control_client = StorageControl::builder()
            .build()
            .await
            .expect("Failed to build test StorageControl client");

        GoogleCloudStorage {
            bucket: format!("projects/_/buckets/{bucket_name}"),
            bucket_name: bucket_name.to_string(),
            client,
            control_client,
        }
    }

    #[tokio::test]
    #[test_log::test]
    async fn builds_url_for_file() {
        let storage = build_test_storage("test-gcs-bucket").await;

        let url = storage
            .get_file_url("uploads/file.pdf")
            .await
            .expect("url should be built");

        assert_eq!(
            url,
            "https://storage.googleapis.com/test-gcs-bucket/uploads/file.pdf"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn builds_url_for_file_with_uuid() {
        let storage = build_test_storage("my-terms-bucket").await;

        let url = storage
            .get_file_url("a1b2c3d4-e5f6-7890-abcd-ef1234567890.pdf")
            .await
            .expect("url should be built");

        assert_eq!(
            url,
            "https://storage.googleapis.com/my-terms-bucket/a1b2c3d4-e5f6-7890-abcd-ef1234567890.pdf"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn builds_url_for_nested_path() {
        let storage = build_test_storage("test-gcs-bucket").await;

        let url = storage
            .get_file_url("terms/2024/file.pdf")
            .await
            .expect("url should be built");

        assert_eq!(
            url,
            "https://storage.googleapis.com/test-gcs-bucket/terms/2024/file.pdf"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn builds_url_with_special_characters() {
        let storage = build_test_storage("test-gcs-bucket").await;

        let url = storage
            .get_file_url("terms/user-123/file%20name.pdf")
            .await
            .expect("url should be built");

        assert_eq!(
            url,
            "https://storage.googleapis.com/test-gcs-bucket/terms/user-123/file%20name.pdf"
        );
    }
}
