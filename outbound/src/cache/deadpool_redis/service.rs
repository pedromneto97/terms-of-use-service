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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use deadpool_redis::redis::{self, AsyncCommands, ConnectionAddr};
    use redis_test::server::RedisServer;
    use std::sync::{Mutex, OnceLock};

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    fn redis_url(server: &RedisServer) -> String {
        match server.client_addr() {
            ConnectionAddr::Tcp(host, port) => format!("redis://{host}:{port}"),
            ConnectionAddr::TcpTls { host, port, .. } => {
                format!("rediss://{host}:{port}")
            }
            ConnectionAddr::Unix(path) => {
                format!("unix://{}", path.to_string_lossy())
            }
        }
    }

    async fn build_cache(
        server: &RedisServer,
        agreement_ttl_seconds: u64,
        term_ttl_seconds: u64,
    ) -> DeadpoolRedisCache {
        let guard = ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env mutex poisoned");

        unsafe {
            std::env::set_var("CACHE_URL", redis_url(server));
            std::env::set_var("AGREEMENT_TTL_SECONDS", agreement_ttl_seconds.to_string());
            std::env::set_var("TERM_TTL_SECONDS", term_ttl_seconds.to_string());
        }

        let cache = DeadpoolRedisCache::new().await;

        unsafe {
            std::env::remove_var("CACHE_URL");
            std::env::remove_var("AGREEMENT_TTL_SECONDS");
            std::env::remove_var("TERM_TTL_SECONDS");
        }

        drop(guard);

        cache
    }

    fn redis_server_available() -> bool {
        std::process::Command::new("redis-server")
            .arg("-v")
            .output()
            .is_ok()
    }

    async fn flushdb(cache: &DeadpoolRedisCache) -> Result<()> {
        let mut conn = cache.get_connection().await?;

        redis::cmd("FLUSHALL")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|err| {
                error!("Failed to flush Redis in tests: {err}");

                TermsOfUseError::InternalServerError
            })
    }

    async fn ttl_for(cache: &DeadpoolRedisCache, key: &str) -> Result<i64> {
        let mut conn = cache.get_connection().await?;

        conn.ttl(key).await.map_err(|err| {
            error!("Failed to fetch TTL in tests: {err}");

            TermsOfUseError::InternalServerError
        })
    }

    fn sample_term(group: &str, version: u32) -> TermOfUse {
        TermOfUse {
            id: version as i32,
            group: group.to_string(),
            url: format!("https://example.com/{group}/{version}"),
            version,
            info: Some("Sample info".to_string()),
            created_at: Utc::now().naive_utc(),
        }
    }

    #[tokio::test]
    async fn new_uses_env_configuration() {
        if !redis_server_available() {
            eprintln!("redis-server not available; skipping test new_uses_env_configuration");
            return;
        }
        let server = RedisServer::new();
        let cache = build_cache(&server, 12, 34).await;

        assert_eq!(cache.agreement_ttl_seconds, 12);
        assert_eq!(cache.term_ttl_seconds, 34);

        let mut conn = cache.get_connection().await.expect("connection");
        let _: String = conn.ping().await.expect("ping should succeed");
    }

    #[tokio::test]
    async fn store_and_find_user_agreement() -> Result<()> {
        if !redis_server_available() {
            eprintln!("redis-server not available; skipping test store_and_find_user_agreement");
            return Ok(());
        }
        let server = RedisServer::new();
        let cache = build_cache(&server, 5, 10).await;
        flushdb(&cache).await?;

        cache
            .store_user_agreement(1, "legal", true)
            .await
            .expect("store should succeed");

        let found = cache.find_user_agreement(1, "legal").await?;
        assert_eq!(found, Some(true));

        let missing = cache.find_user_agreement(2, "legal").await?;
        assert!(missing.is_none());

        let ttl = ttl_for(&cache, &format!("{USER_AGREEMENTS_PREFIX}legal:1")).await?;
        assert!(ttl <= 5 && ttl > 0);

        Ok(())
    }

    #[tokio::test]
    async fn store_and_get_latest_term_for_group() -> Result<()> {
        if !redis_server_available() {
            eprintln!(
                "redis-server not available; skipping test store_and_get_latest_term_for_group"
            );
            return Ok(());
        }
        let server = RedisServer::new();
        let cache = build_cache(&server, 5, 8).await;
        flushdb(&cache).await?;

        let term = sample_term("group-a", 2);
        cache.store_latest_term_for_group(&term).await?;

        let fetched = cache.get_latest_term_for_group("group-a").await?;
        let latest = fetched.expect("term should exist");

        assert_eq!(latest.group, term.group);
        assert_eq!(latest.url, term.url);
        assert_eq!(latest.version, term.version);
        assert_eq!(latest.info, term.info);
        assert_eq!(latest.created_at, term.created_at);

        let ttl = ttl_for(&cache, &format!("{LATEST_TERMS_PREFIX}{}", term.group)).await?;
        assert!(ttl <= 8 && ttl > 0);

        Ok(())
    }

    #[tokio::test]
    async fn get_latest_term_for_group_returns_none_when_absent() -> Result<()> {
        if !redis_server_available() {
            eprintln!(
                "redis-server not available; skipping test get_latest_term_for_group_returns_none_when_absent"
            );
            return Ok(());
        }
        let server = RedisServer::new();
        let cache = build_cache(&server, 5, 5).await;
        flushdb(&cache).await?;

        let fetched = cache.get_latest_term_for_group("missing").await?;

        assert!(fetched.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn invalidate_cache_for_group_removes_related_keys() -> Result<()> {
        if !redis_server_available() {
            eprintln!(
                "redis-server not available; skipping test invalidate_cache_for_group_removes_related_keys"
            );
            return Ok(());
        }
        let server = RedisServer::new();
        let cache = build_cache(&server, 10, 10).await;
        flushdb(&cache).await?;

        cache.store_user_agreement(1, "group-a", true).await?;
        cache.store_user_agreement(2, "group-a", false).await?;
        cache
            .store_latest_term_for_group(&sample_term("group-a", 1))
            .await?;

        cache.store_user_agreement(1, "group-b", true).await?;
        cache
            .store_latest_term_for_group(&sample_term("group-b", 1))
            .await?;

        cache.invalidate_cache_for_group("group-a").await?;

        let mut conn = cache.get_connection().await?;
        let group_a_one: i64 = conn
            .exists(format!("{USER_AGREEMENTS_PREFIX}group-a:1"))
            .await
            .map_err(|err| {
                error!("Failed to check key existence: {err}");

                TermsOfUseError::InternalServerError
            })?;
        let group_a_two: i64 = conn
            .exists(format!("{USER_AGREEMENTS_PREFIX}group-a:2"))
            .await
            .map_err(|err| {
                error!("Failed to check key existence: {err}");

                TermsOfUseError::InternalServerError
            })?;
        let group_a_term: i64 = conn
            .exists(format!("{LATEST_TERMS_PREFIX}group-a"))
            .await
            .map_err(|err| {
                error!("Failed to check key existence: {err}");

                TermsOfUseError::InternalServerError
            })?;

        assert_eq!(group_a_one + group_a_two + group_a_term, 0);

        let group_b_term: i64 = conn
            .exists(format!("{LATEST_TERMS_PREFIX}group-b"))
            .await
            .map_err(|err| {
                error!("Failed to check key existence: {err}");

                TermsOfUseError::InternalServerError
            })?;

        assert_eq!(group_b_term, 1);

        Ok(())
    }
}
