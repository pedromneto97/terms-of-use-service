use std::path::Path;

use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Builder as S3ConfigBuilder, primitives::ByteStream};
use domain::{data::service::StorageService, errors::TermsOfUseError};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct StorageConfig {
    bucket_name: String,
    client: aws_sdk_s3::Client,
    endpoint_url: Option<String>,
}

impl StorageConfig {
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

        StorageConfig {
            bucket_name,
            client,
            endpoint_url,
        }
    }
}

impl StorageService for StorageConfig {
    async fn upload_file(
        &self,
        path: &Path,
        content_type: &str,
    ) -> Result<String, TermsOfUseError> {
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

    async fn delete_file(&self, path: &str) -> Result<(), TermsOfUseError> {
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

    async fn get_file_url(&self, path: &str) -> Result<String, TermsOfUseError> {
        if let Some(ref endpoint_url) = self.endpoint_url {
            return Ok(format!("{}/{}/{}", endpoint_url, self.bucket_name, path));
        }

        Ok(format!(
            "https://{}.s3.amazonaws.com/{path}",
            self.bucket_name,
        ))
    }
}
