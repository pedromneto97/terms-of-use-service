#[cfg(feature = "postgres")]
pub type AppRepository = crate::outbound::database::postgres::PostgresRepository;
