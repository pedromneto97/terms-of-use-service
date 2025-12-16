use std::path::Path;

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Builder as S3ConfigBuilder, primitives::ByteStream};
use domain::{
    data::service::StorageService,
    errors::{Result, TermsOfUseError},
};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct S3Storage {
    bucket_name: String,
    client: aws_sdk_s3::Client,
    endpoint_url: Option<String>,
}

impl S3Storage {
    pub async fn new() -> Self {
        let config_builder = aws_config::defaults(BehaviorVersion::latest());

        let endpoint_url = std::env::var("AWS_ENDPOINT_URL").ok();

        let config = if let Some(ref url) = endpoint_url {
            info!("Using custom S3 endpoint URL: {url}");

            config_builder.endpoint_url(url).load().await
        } else {
            config_builder.load().await
        };

        let bucket_name =
            std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set in env vars");

        // Build S3 client with path-style addressing for LocalStack/MinIO compatibility
        let s3_config_builder = S3ConfigBuilder::from(&config);

        let s3_config = if endpoint_url.is_some() {
            // Use path-style for local development (LocalStack, MinIO, etc.)
            s3_config_builder.force_path_style(true).build()
        } else {
            s3_config_builder.build()
        };

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        S3Storage {
            bucket_name,
            client,
            endpoint_url,
        }
    }
}

#[async_trait]
impl StorageService for S3Storage {
    async fn upload_file(&self, path: &Path, content_type: &str) -> Result<String> {
        let body = ByteStream::from_path(path).await.map_err(|err| {
            error!("Failed to read file for upload: {err}");

            TermsOfUseError::InternalServerError
        })?;

        let file_extension =
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or(match content_type {
                    "application/pdf" => "pdf",
                    _ => "",
                });

        let key = format!("{}.{file_extension}", uuid::Uuid::new_v4());

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to upload file to S3: {err}");
                error!("Bucket: {}, Key: {}", &self.bucket_name, &key);
                error!("Content-Type: {:?}", err.into_source());

                TermsOfUseError::InternalServerError
            })?;

        Ok(key)
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(path)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to delete file from S3: {err}");

                TermsOfUseError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_file_url(&self, path: &str) -> Result<String> {
        if let Some(ref endpoint_url) = self.endpoint_url {
            return Ok(format!("{}/{}/{}", endpoint_url, self.bucket_name, path));
        }

        Ok(format!(
            "https://{}.s3.amazonaws.com/{path}",
            self.bucket_name,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
    use aws_types::region::Region;
    use tokio::fs;

    fn build_storage(endpoint_url: Option<&str>) -> S3Storage {
        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
            .region(Region::new("us-east-1"))
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        S3Storage {
            bucket_name: "test-bucket".to_string(),
            client: client,
            endpoint_url: endpoint_url.map(String::from),
        }
    }

    #[tokio::test]
    async fn builds_url_with_custom_endpoint() {
        let storage = build_storage(Some("http://localhost:4566"));

        let url = storage
            .get_file_url("uploads/file.pdf")
            .await
            .expect("url should be built");

        assert_eq!(url, "http://localhost:4566/test-bucket/uploads/file.pdf");
    }

    #[tokio::test]
    async fn builds_url_with_default_host() {
        let storage = build_storage(None);

        let url = storage
            .get_file_url("uploads/file.pdf")
            .await
            .expect("url should be built");

        assert_eq!(url, "https://test-bucket.s3.amazonaws.com/uploads/file.pdf");
    }

    #[tokio::test]
    async fn upload_file_fails_without_real_s3() {
        let storage = build_storage(None);

        let temp_file = std::env::temp_dir().join("test-upload-fail.txt");
        fs::write(&temp_file, "test content").await.unwrap();

        let result = storage.upload_file(&temp_file, "text/plain").await;

        fs::remove_file(&temp_file).await.ok();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_upload_file_successfully() {
        let storage = S3Storage::new().await;

        // Create bucket first
        storage
            .client
            .create_bucket()
            .bucket(&storage.bucket_name)
            .send()
            .await
            .unwrap();

        // Create a temporary test file
        let temp_file = std::env::temp_dir().join("test-upload.txt");
        fs::write(&temp_file, b"Hello S3!").await.unwrap();

        // Upload file
        let result = storage.upload_file(&temp_file, "text/plain").await.unwrap();

        assert!(
            result.ends_with(".txt"),
            "Key should end with .txt extension"
        );

        // Verify file exists in S3
        let _ = storage
            .client
            .head_object()
            .bucket(&storage.bucket_name)
            .key(&result)
            .send()
            .await
            .unwrap();

        // Clean up
        fs::remove_file(&temp_file).await.ok();
        storage.delete_file(&result).await.ok();
    }

    #[tokio::test]
    async fn should_upload_pdf_file_successfully() {
        let storage = S3Storage::new().await;

        // Create bucket first
        storage
            .client
            .create_bucket()
            .bucket(&storage.bucket_name)
            .send()
            .await
            .ok();

        // Create a temporary test PDF file
        let temp_file = std::env::temp_dir().join("test-upload.pdf");

        fs::write(&temp_file, "%PDF-1.4 test content")
            .await
            .unwrap();

        // Upload file
        let result = storage
            .upload_file(&temp_file, "application/pdf")
            .await
            .unwrap();

        assert!(
            result.ends_with(".pdf"),
            "Key should end with .pdf extension"
        );

        // Clean up
        fs::remove_file(&temp_file).await.ok();
        storage.delete_file(&result).await.ok();
    }

    #[tokio::test]
    async fn should_delete_file_successfully() {
        let storage = S3Storage::new().await;

        // Create bucket first
        storage
            .client
            .create_bucket()
            .bucket(&storage.bucket_name)
            .send()
            .await
            .ok();

        // Create and upload a test file
        let temp_file = std::env::temp_dir().join("test-delete.txt");
        fs::write(&temp_file, "test content for deletion")
            .await
            .unwrap();

        let key = storage.upload_file(&temp_file, "text/plain").await.unwrap();

        storage.delete_file(&key).await.unwrap();

        // Verify file no longer exists
        let head_result = storage
            .client
            .head_object()
            .bucket(&storage.bucket_name)
            .key(&key)
            .send()
            .await;

        assert!(
            head_result.is_err(),
            "File should not exist in S3 after deletion"
        );

        // Clean up
        fs::remove_file(&temp_file).await.ok();
    }
}
