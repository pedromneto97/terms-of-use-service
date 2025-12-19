use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use super::DeadpoolRedisCache;

#[async_trait]
impl HealthCheck for DeadpoolRedisCache {
    async fn ping(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        conn.ping().await.map_err(|err| {
            error!("Failed to ping Redis cache: {err}");

            TermsOfUseError::InternalServerError
        })
    }
}

#[cfg(test)]
mod tests {
    use domain::data::health_check::HealthCheck;

    use crate::cache::deadpool_redis::tests::{build_cache, redis_server_available};

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_succeed_with_valid_connection() {
        if !redis_server_available() {
            return; // Skip if redis-server is not available
        }

        let server = redis_test::server::RedisServer::new();
        let cache = build_cache(&server, 60, 3600).await;

        let result = cache.ping().await;

        assert!(
            result.is_ok(),
            "ping should succeed with valid Redis connection"
        );
    }
}
