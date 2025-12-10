#[cfg(all(feature = "dynamodb", feature = "postgres"))]
compile_error!("Features 'dynamodb' and 'postgres' cannot be enabled at the same time.");

#[cfg(not(any(feature = "dynamodb", feature = "postgres")))]
compile_error!("Either feature 'dynamodb' or 'postgres' must be enabled.");

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "dynamodb")]
pub mod dynamodb;
