use deadpool_redis::{Connection, Runtime, redis::AsyncCommands};
use log::error;

use crate::domain::{data::service::CacheService, entities::TermOfUse, errors::TermsOfUseError};

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pool: deadpool_redis::Pool,
    agreement_ttl_seconds: u64,
    term_ttl_seconds: u64,
}

impl RedisConfig {
    pub async fn new() -> Self {
        let url = std::env::var("REDIS_URL").expect("REDIS_URL must be set in env vars");

        let agreement_ttl_seconds = std::env::var("AGREEMENT_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .expect("AGREEMENT_TTL_SECONDS must be a valid usize");

        let term_ttl_seconds = std::env::var("TERM_TTL_SECONDS")
            .unwrap_or_else(|_| "86400".to_string()) // One day in seconds
            .parse()
            .expect("TERM_TTL_SECONDS must be a valid usize");

        let config = deadpool_redis::Config::from_url(url);

        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create Redis pool");

        RedisConfig {
            pool,
            agreement_ttl_seconds,
            term_ttl_seconds,
        }
    }

    async fn get_connection(&self) -> Result<Connection, TermsOfUseError> {
        self.pool.get().await.map_err(|err| {
            error!("Failed to get Redis connection: {err}");

            TermsOfUseError::InternalServerError
        })
    }
}

const USER_AGREEMENTS_PREFIX: &str = "USER_AGREEMENTS:";
const LATEST_TERMS_PREFIX: &str = "LATEST_TERMS:";

impl CacheService for RedisConfig {
    async fn find_user_agreement(
        &self,
        user_id: i32,
        group: &str,
    ) -> Result<Option<bool>, TermsOfUseError> {
        let mut conn = self.get_connection().await?;

        let key = format!("{USER_AGREEMENTS_PREFIX}{group}:{user_id}");

        conn.get::<String, Option<bool>>(key).await.map_err(|err| {
            error!("Failed to get user agreement from cache: {err}");

            TermsOfUseError::InternalServerError
        })
    }

    async fn store_user_agreement(
        &self,
        user_id: i32,
        group: &str,
        agreed: bool,
    ) -> Result<(), TermsOfUseError> {
        let mut conn = self.get_connection().await?;

        let key = format!("{USER_AGREEMENTS_PREFIX}{group}:{user_id}");

        conn.set_ex::<String, bool, ()>(key, agreed, self.agreement_ttl_seconds)
            .await
            .map_err(|err| {
                error!("Failed to store user agreement in cache: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    async fn get_latest_term_for_group(
        &self,
        group: &str,
    ) -> Result<Option<TermOfUse>, TermsOfUseError> {
        let mut conn = self.get_connection().await?;

        let key = format!("{LATEST_TERMS_PREFIX}{group}");

        let result = conn
            .get::<String, Option<String>>(key)
            .await
            .map_err(|err| {
                error!("Failed to get latest term from cache: {err}");

                TermsOfUseError::InternalServerError
            })?;

        match result {
            Some(serialized_term) => {
                let term: TermOfUse = serde_json::from_str(&serialized_term).map_err(|err| {
                    error!("Failed to deserialize term from cache: {err}");

                    TermsOfUseError::InternalServerError
                })?;

                Ok(Some(term))
            }
            None => Ok(None),
        }
    }

    async fn store_latest_term_for_group(&self, term: &TermOfUse) -> Result<(), TermsOfUseError> {
        let mut conn = self.get_connection().await?;

        let key = format!("{LATEST_TERMS_PREFIX}{}", term.group);
        let value = serde_json::to_string(term).map_err(|err| {
            error!("Failed to serialize term for caching: {err}");

            TermsOfUseError::InternalServerError
        })?;

        conn.set_ex::<String, String, ()>(key, value, self.term_ttl_seconds)
            .await
            .map_err(|err| {
                error!("Failed to store latest term in cache: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    async fn invalidate_cache_for_group(&self, group: &str) -> Result<(), TermsOfUseError> {
        let mut conn = self.get_connection().await?;

        let keys = [
            format!("{LATEST_TERMS_PREFIX}{group}"),
            format!("{USER_AGREEMENTS_PREFIX}{group}:*"),
        ];

        conn.del::<&[String], ()>(&keys).await.map_err(|err| {
            error!("Failed to invalidate cache for group {group}: {err}");

            TermsOfUseError::InternalServerError
        })
    }
}
