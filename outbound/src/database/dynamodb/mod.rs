use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use domain::errors::TermsOfUseError;
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
    async fn get_next_id(&self, counter_name: &str) -> Result<i32, TermsOfUseError> {
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

#[cfg(test)]
mod tests {
    use crate::database::dynamodb::{
        DynamoRepository,
        migration::COUNTERS_TABLE,
        model::{TERMS_TABLE, USER_AGREEMENTS_TABLE},
    };
    use aws_sdk_dynamodb::types::AttributeValue;
    use tokio::runtime::{Handle, Runtime};

    impl Drop for DynamoRepository {
        fn drop(&mut self) {
            let client = self.client.clone();

            let cleanup = async move {
                if let Ok(scan_res) = client.scan().table_name(TERMS_TABLE).send().await {
                    if let Some(items) = scan_res.items {
                        for item in items {
                            if let Some(id_str) = item.get("id").and_then(|v| v.as_n().ok()) {
                                let _ = client
                                    .delete_item()
                                    .table_name(TERMS_TABLE)
                                    .key("id", AttributeValue::N(id_str.to_string()))
                                    .send()
                                    .await;
                            }
                        }
                    }
                }

                if let Ok(scan_res) = client.scan().table_name(COUNTERS_TABLE).send().await {
                    if let Some(items) = scan_res.items {
                        for item in items {
                            if let Some(name) = item.get("counter_name").and_then(|v| v.as_s().ok())
                            {
                                let _ = client
                                    .delete_item()
                                    .table_name(COUNTERS_TABLE)
                                    .key("counter_name", AttributeValue::S(name.to_string()))
                                    .send()
                                    .await;
                            }
                        }
                    }
                }

                if let Ok(scan_res) = client.scan().table_name(USER_AGREEMENTS_TABLE).send().await {
                    if let Some(items) = scan_res.items {
                        for item in items {
                            if let Some(agreement) =
                                item.get("agreement_key").and_then(|v| v.as_s().ok())
                            {
                                let _ = client
                                    .delete_item()
                                    .table_name(USER_AGREEMENTS_TABLE)
                                    .key("agreement_key", AttributeValue::S(agreement.to_string()))
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
            };

            if let Ok(handle) = Handle::try_current() {
                handle.spawn(cleanup);
            } else if let Ok(rt) = Runtime::new() {
                let _ = rt.block_on(cleanup);
            }
        }
    }
}
