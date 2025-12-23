use async_trait::async_trait;
use domain::{
    data::health_check::HealthCheck,
    errors::{Result, TermsOfUseError},
};
use tracing::error;

use super::super::PostgresRepository;

#[async_trait]
impl HealthCheck for PostgresRepository {
    async fn ping(&self) -> Result<()> {
        self.db.ping().await.map_err(|err| {
            error!("Failed to ping Postgres database: {err}");

            TermsOfUseError::InternalServerError
        })
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DatabaseBackend, MockDatabase};

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn health_check_ping_should_succeed_with_valid_connection() {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

        let repository = PostgresRepository::from_connection(db);

        let result = repository.ping().await;

        assert!(
            result.is_ok(),
            "ping should succeed with valid Postgres connection"
        );
    }
}
