use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use domain::data::StorageServiceWithHealthCheck;
use tracing::info;

mod health_check;
mod service;

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

impl StorageServiceWithHealthCheck for S3Storage {}
