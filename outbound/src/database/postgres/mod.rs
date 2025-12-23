use domain::data::{DatabaseRepositoryWithHealthCheck, repository::DatabaseRepository};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

mod data;

#[derive(Clone, Debug)]
pub struct PostgresRepository {
    pub db: DatabaseConnection,
}

impl PostgresRepository {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment variables");
        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .expect("DATABASE_MAX_CONNECTIONS must be a valid u32");
        let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .expect("DATABASE_MIN_CONNECTIONS must be a valid u32");
        let max_connection_lifetime_secs = std::env::var("DATABASE_MAX_CONNECTION_LIFETIME_SECS")
            .unwrap_or_default()
            .parse::<u64>()
            .ok();

        let mut connection_options = sea_orm::ConnectOptions::new(database_url.clone());

        connection_options
            .max_connections(max_connections)
            .min_connections(min_connections);

        if let Some(lifetime) = max_connection_lifetime_secs {
            connection_options.max_lifetime(std::time::Duration::from_secs(lifetime));
        }

        let database = Database::connect(connection_options).await.unwrap();

        Migrator::up(&database, None).await.unwrap();

        Self { db: database }
    }

    #[cfg(test)]
    pub fn from_connection(database: DatabaseConnection) -> Self {
        Self { db: database }
    }
}

impl DatabaseRepository for PostgresRepository {}

impl DatabaseRepositoryWithHealthCheck for PostgresRepository {}
