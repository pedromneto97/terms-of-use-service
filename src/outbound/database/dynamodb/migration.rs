use aws_sdk_dynamodb::types::{
    AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ScalarAttributeType,
};
use domain::errors::{Result, TermsOfUseError};
use tracing::{error, info};

use crate::outbound::database::dynamodb::model::{TERMS_TABLE, USER_AGREEMENTS_TABLE};

pub const GSI_TERMS_GROUP_VERSION: &str = "gsi_group_version";
pub const COUNTERS_TABLE: &str = "counters";

pub async fn run_migrations(client: &aws_sdk_dynamodb::Client) -> Result<()> {
    create_counters_table(client).await?;
    create_terms_table(client).await?;
    create_user_agreements_table(client).await?;

    Ok(())
}

async fn table_exists(client: &aws_sdk_dynamodb::Client, table_name: &str) -> bool {
    client
        .describe_table()
        .table_name(table_name)
        .send()
        .await
        .is_ok()
}

/// Creates the `terms` table with:
/// - Primary key: `id` (Number)
/// - Global secondary index `gsi_group_version`: partition key `group` (String), sort key `version` (Number)
async fn create_terms_table(client: &aws_sdk_dynamodb::Client) -> Result<()> {
    if table_exists(client, TERMS_TABLE).await {
        info!("Table '{TERMS_TABLE}' already exists, skipping creation");

        return Ok(());
    }

    let id_attr = build_attribute_definition("id", ScalarAttributeType::N)?;
    let group_attr = build_attribute_definition("group", ScalarAttributeType::S)?;
    let version_attr = build_attribute_definition("version", ScalarAttributeType::N)?;

    let pk_schema = build_key_schema_element("id", KeyType::Hash)?;

    let gsi_group_version = GlobalSecondaryIndex::builder()
        .index_name(GSI_TERMS_GROUP_VERSION)
        .key_schema(build_key_schema_element("group", KeyType::Hash)?)
        .key_schema(build_key_schema_element("version", KeyType::Range)?)
        .projection(
            Projection::builder()
                .projection_type(ProjectionType::All)
                .build(),
        )
        .build()
        .map_err(|err| {
            error!("Failed to build GSI '{GSI_TERMS_GROUP_VERSION}': {err}");

            TermsOfUseError::InternalServerError
        })?;

    client
        .create_table()
        .table_name(TERMS_TABLE)
        .attribute_definitions(id_attr)
        .attribute_definitions(group_attr)
        .attribute_definitions(version_attr)
        .key_schema(pk_schema)
        .global_secondary_indexes(gsi_group_version)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await
        .map_err(|err| {
            error!("Failed to create DynamoDB table '{TERMS_TABLE}': {err}");

            TermsOfUseError::InternalServerError
        })?;

    info!("Created DynamoDB table '{TERMS_TABLE}'");

    Ok(())
}

/// Creates the `user_agreements` table with:
/// - Primary key: `agreement_key` (String) - Format: "{user_id}#{term_id}"
/// This design allows direct GetItem queries without needing a GSI
async fn create_user_agreements_table(client: &aws_sdk_dynamodb::Client) -> Result<()> {
    if table_exists(client, USER_AGREEMENTS_TABLE).await {
        info!("Table '{USER_AGREEMENTS_TABLE}' already exists, skipping creation");

        return Ok(());
    }

    let agreement_key_attr = build_attribute_definition("agreement_key", ScalarAttributeType::S)?;
    let pk_schema = build_key_schema_element("agreement_key", KeyType::Hash)?;

    client
        .create_table()
        .table_name(USER_AGREEMENTS_TABLE)
        .attribute_definitions(agreement_key_attr)
        .key_schema(pk_schema)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await
        .map_err(|err| {
            error!("Failed to create DynamoDB table '{USER_AGREEMENTS_TABLE}': {err}");

            TermsOfUseError::InternalServerError
        })?;

    info!("Created DynamoDB table '{USER_AGREEMENTS_TABLE}'");

    Ok(())
}

fn build_attribute_definition(
    name: &str,
    attr_type: ScalarAttributeType,
) -> Result<AttributeDefinition> {
    AttributeDefinition::builder()
        .attribute_name(name)
        .attribute_type(attr_type)
        .build()
        .map_err(|err| {
            error!("Failed to build attribute definition for '{name}': {err}");

            TermsOfUseError::InternalServerError
        })
}

fn build_key_schema_element(name: &str, key_type: KeyType) -> Result<KeySchemaElement> {
    KeySchemaElement::builder()
        .attribute_name(name)
        .key_type(key_type)
        .build()
        .map_err(|err| {
            error!("Failed to build key schema element for '{name}': {err}");

            TermsOfUseError::InternalServerError
        })
}

/// Creates the `counters` table for atomic ID generation:
/// - Primary key: `counter_name` (String)
/// - Attribute: `current_value` (Number)
async fn create_counters_table(client: &aws_sdk_dynamodb::Client) -> Result<(), TermsOfUseError> {
    if table_exists(client, COUNTERS_TABLE).await {
        info!("Table '{COUNTERS_TABLE}' already exists, skipping creation");

        return Ok(());
    }

    let counter_name_attr = build_attribute_definition("counter_name", ScalarAttributeType::S)?;
    let pk_schema = build_key_schema_element("counter_name", KeyType::Hash)?;

    client
        .create_table()
        .table_name(COUNTERS_TABLE)
        .attribute_definitions(counter_name_attr)
        .key_schema(pk_schema)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await
        .map_err(|err| {
            error!("Failed to create DynamoDB table '{COUNTERS_TABLE}': {err}");

            TermsOfUseError::InternalServerError
        })?;

    info!("Created DynamoDB table '{COUNTERS_TABLE}'");

    Ok(())
}
