use deadpool_redis::{Connection, Pool, Runtime};
use domain::errors::{Result, TermsOfUseError};
use tracing::error;

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
