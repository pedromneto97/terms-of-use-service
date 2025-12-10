#[cfg(feature = "postgres")]
pub type AppRepository = crate::outbound::database::postgres::PostgresRepository;

#[cfg(feature = "dynamodb")]
pub type AppRepository = crate::outbound::database::dynamodb::DynamoDBClient;
