#[cfg(not(any(feature = "s3", feature = "gcloud", test)))]
compile_error!("No storage feature enabled. Please enable at least one: 's3' or 'gcloud'.");

#[cfg(all(feature = "s3", feature = "gcloud", not(test)))]
compile_error!("Multiple storage features enabled. Please enable only one: 's3' or 'gcloud'.");

#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "gcloud")]
pub mod gcloud;
