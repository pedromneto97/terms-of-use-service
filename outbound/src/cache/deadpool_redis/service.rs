use deadpool_redis::redis::{AsyncCommands, pipe};
use domain::{
    data::service::CacheService,
    entities::TermOfUse,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use crate::cache::deadpool_redis::DeadpoolRedisCache;

const USER_AGREEMENTS_PREFIX: &str = "USER_AGREEMENTS:";
const LATEST_TERMS_PREFIX: &str = "LATEST_TERMS:";

impl CacheService for DeadpoolRedisCache {
    async fn find_user_agreement(&self, user_id: i32, group: &str) -> Result<Option<bool>> {
        let mut conn = self.get_connection().await?;

        let key = format!("{USER_AGREEMENTS_PREFIX}{group}:{user_id}");

        conn.get::<String, Option<bool>>(key).await.map_err(|err| {
            error!("Failed to get user agreement from cache: {err}");

            TermsOfUseError::InternalServerError
        })
    }

    async fn store_user_agreement(&self, user_id: i32, group: &str, agreed: bool) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let key = format!("{USER_AGREEMENTS_PREFIX}{group}:{user_id}");

        conn.set_ex::<String, bool, ()>(key, agreed, self.agreement_ttl_seconds)
            .await
            .map_err(|err| {
                error!("Failed to store user agreement in cache: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    async fn get_latest_term_for_group(&self, group: &str) -> Result<Option<TermOfUse>> {
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

    async fn store_latest_term_for_group(&self, term: &TermOfUse) -> Result<()> {
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

    async fn invalidate_cache_for_group(&self, group: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let mut pipe = pipe();
        pipe.atomic();

        let mut keys = conn
            .scan_match::<String, String>(format!("{USER_AGREEMENTS_PREFIX}{group}:*"))
            .await
            .map_err(|err| {
                error!("Failed to scan keys for cache invalidation: {err}");

                TermsOfUseError::InternalServerError
            })?;

        while let Some(key) = keys.next_item().await {
            pipe.unlink(key).ignore();
        }
        drop(keys);

        pipe.unlink(format!("{LATEST_TERMS_PREFIX}{group}"))
            .ignore();

        pipe.query_async::<()>(&mut conn).await.map_err(|err| {
            error!("Failed to invalidate cache for group: {err}");

            TermsOfUseError::InternalServerError
        })
    }
}
