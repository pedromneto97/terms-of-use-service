use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use domain::{
    data::{
        DatabaseRepositoryWithHealthCheck, health_check::HealthCheck,
        repository::DatabaseRepository,
    },
    errors::{Result, TermsOfUseError},
};
use tracing::{error, info};

mod migration;
mod model;
mod repositories;

#[derive(Clone, Debug)]
pub struct DynamoRepository {
    client: aws_sdk_dynamodb::Client,
}

impl DynamoRepository {
    pub async fn new() -> Self {
        let config_builder = aws_config::defaults(BehaviorVersion::latest());

        let endpoint_url = std::env::var("AWS_ENDPOINT_URL").ok();

        let config = if let Some(ref url) = endpoint_url {
            info!("Using custom DynamoDB endpoint URL: {url}");

            config_builder.endpoint_url(url).load().await
        } else {
            config_builder.load().await
        };

        let config_builder = aws_sdk_dynamodb::config::Builder::from(&config).build();

        let client = aws_sdk_dynamodb::Client::from_conf(config_builder);

        migration::run_migrations(&client)
            .await
            .expect("Failed to run migrations");

        Self { client }
    }

    /// Atomically increments and returns the next ID for a given counter
    async fn get_next_id(&self, counter_name: &str) -> Result<i32> {
        let result = self
            .client
            .update_item()
            .table_name(migration::COUNTERS_TABLE)
            .key("counter_name", AttributeValue::S(counter_name.to_string()))
            .update_expression("ADD current_value :incr")
            .expression_attribute_values(":incr", AttributeValue::N("1".to_string()))
            .return_values(ReturnValue::UpdatedNew)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to get next ID for counter '{counter_name}': {err}");

                TermsOfUseError::InternalServerError
            })?;

        let current_value = result
            .attributes
            .and_then(|attrs| attrs.get("current_value").cloned())
            .ok_or_else(|| {
                error!("Counter '{counter_name}' did not return current_value");

                TermsOfUseError::InternalServerError
            })?;

        let value_str = current_value.as_n().map_err(|_| {
            error!("Counter '{counter_name}' returned non-numeric value");

            TermsOfUseError::InternalServerError
        })?;

        value_str.parse::<i32>().map_err(|err| {
            error!("Failed to parse counter value for '{counter_name}': {err}");

            TermsOfUseError::InternalServerError
        })
    }
}

impl DatabaseRepository for DynamoRepository {}

#[async_trait]
impl HealthCheck for DynamoRepository {
    async fn ping(&self) -> Result<()> {
        self.client
            .list_tables()
            .limit(1)
            .send()
            .await
            .map_err(|err| {
                error!("Failed to ping DynamoDB database: {err}");

                TermsOfUseError::InternalServerError
            })
            .map(|_| ())
    }
}

#[async_trait]
impl DatabaseRepositoryWithHealthCheck for DynamoRepository {}
