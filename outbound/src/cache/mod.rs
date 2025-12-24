#[cfg(any(not(feature = "cache"), clippy, rustfmt, test))]
pub mod noop;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "valkey")]
pub mod valkey;

#[cfg(feature = "deadpool-redis")]
mod deadpool_redis;
