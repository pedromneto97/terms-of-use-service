use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use chrono::DateTime;
use domain::{
    entities::TermOfUse,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

pub const TERMS_TABLE: &str = "terms";
pub const USER_AGREEMENTS_TABLE: &str = "user_agreements";

fn as_string(val: Option<&AttributeValue>) -> String {
    if let Some(v) = val
        && let Ok(s) = v.as_s()
    {
        return s.to_owned();
    }

    "".to_owned()
}

fn as_optional_string(val: Option<&AttributeValue>) -> Option<String> {
    if let Some(v) = val
        && let Ok(s) = v.as_s()
    {
        return Some(s.to_owned());
    }

    None
}

fn as_i32(val: Option<&AttributeValue>) -> i32 {
    if let Some(v) = val
        && let Ok(s) = v.as_n()
        && let Ok(n) = s.parse::<i32>()
    {
        return n;
    }

    0
}

fn as_i64(val: Option<&AttributeValue>) -> i64 {
    if let Some(v) = val
        && let Ok(s) = v.as_n()
        && let Ok(n) = s.parse::<i64>()
    {
        return n;
    }

    0
}

fn as_u32(val: Option<&AttributeValue>) -> u32 {
    if let Some(v) = val
        && let Ok(s) = v.as_n()
        && let Ok(n) = s.parse::<u32>()
    {
        return n;
    }

    0
}

pub fn map_term_from_item(item: &HashMap<String, AttributeValue>) -> Result<TermOfUse> {
    let id = as_i32(item.get("id"));
    let group = as_string(item.get("group"));
    let version = as_u32(item.get("version"));
    let url = as_string(item.get("url"));
    let info = as_optional_string(item.get("info"));
    let timestamp = as_i64(item.get("created_at"));
    let created_at = DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| {
            error!("Failed to parse created_at timestamp: {timestamp}");

            TermsOfUseError::InternalServerError
        })?
        .naive_utc();

    Ok(TermOfUse {
        id,
        group,
        version,
        url,
        info,
        created_at,
    })
}
