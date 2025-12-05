mod config;

#[cfg(feature = "otel")]
pub mod telemetry;

pub use config::Config;
