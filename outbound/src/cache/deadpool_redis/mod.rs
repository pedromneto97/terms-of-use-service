use deadpool_redis::{Connection, Pool, Runtime};
use domain::{
    data::CacheServiceWithHealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

mod health_check;
mod service;

#[derive(Clone, Debug)]
pub struct DeadpoolRedisCache {
    pool: Pool,
    agreement_ttl_seconds: u64,
    term_ttl_seconds: u64,
}

impl DeadpoolRedisCache {
    pub async fn new() -> Self {
        let url = std::env::var("CACHE_URL").expect("CACHE_URL must be set in env vars");

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

        DeadpoolRedisCache {
            pool,
            agreement_ttl_seconds,
            term_ttl_seconds,
        }
    }

    async fn get_connection(&self) -> Result<Connection> {
        self.pool.get().await.map_err(|err| {
            error!("Failed to get Redis connection: {err}");

            TermsOfUseError::InternalServerError
        })
    }
}

impl CacheServiceWithHealthCheck for DeadpoolRedisCache {}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use deadpool_redis::redis::{self, ConnectionAddr};
    use domain::errors::{Result, TermsOfUseError};
    use redis_test::server::RedisServer;
    use tracing::error;

    use super::DeadpoolRedisCache;

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    pub fn redis_url(server: &RedisServer) -> String {
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

    pub fn redis_server_available() -> bool {
        std::process::Command::new("redis-server")
            .arg("-v")
            .output()
            .is_ok()
    }

    pub async fn build_cache(
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

    pub async fn flushdb(cache: &DeadpoolRedisCache) -> Result<()> {
        let mut conn = cache.get_connection().await?;

        redis::cmd("FLUSHALL")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|err| {
                error!("Failed to flush Redis in tests: {err}");

                TermsOfUseError::InternalServerError
            })
    }
}
