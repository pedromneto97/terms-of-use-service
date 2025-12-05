use std::path::Path;

use aws_config::BehaviorVersion;
use aws_sdk_s3::primitives::ByteStream;

use crate::domain::{data::service::StorageService, errors::TermsOfUseError};

#[derive(Clone, Debug)]
pub struct StorageConfig {
    bucket_name: String,
    client: aws_sdk_s3::Client,
}

impl StorageConfig {
    pub async fn new() -> Self {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let bucket_name =
            std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set in env vars");

        let client = aws_sdk_s3::Client::new(&config);

        StorageConfig {
            bucket_name,
            client,
        }
    }
}

impl StorageService for StorageConfig {
    async fn upload_file(&self, path: &Path) -> Result<String, TermsOfUseError> {
        let body = ByteStream::from_path(path)
            .await
            .map_err(|_| TermsOfUseError::InternalServerError)?;
        let file_extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(TermsOfUseError::InternalServerError)?;

        let key = format!("{}.{file_extension}", uuid::Uuid::new_v4());

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .send()
            .await
            .map_err(|_| TermsOfUseError::InternalServerError)?;

        Ok(key)
    }

    async fn delete_file(&self, path: &str) -> Result<(), TermsOfUseError> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(path)
            .send()
            .await
            .map_err(|_| TermsOfUseError::InternalServerError)?;

        Ok(())
    }

    async fn get_file_url(&self, path: &str) -> Result<String, TermsOfUseError> {
        Ok(format!(
            "https://{}.s3.amazonaws.com/{path}",
            self.bucket_name,
        ))
    }
}
