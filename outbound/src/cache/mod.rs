#[cfg(not(feature = "cache"))]
pub mod noop;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "valkey")]
pub mod valkey;

#[cfg(feature = "deadpool-redis")]
mod deadpool_redis;
