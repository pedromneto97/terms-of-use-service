#[cfg(not(feature = "cache"))]
pub mod noop;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "valkey")]
pub mod valkey;

#[cfg(any(feature = "deadpool-redis", test))]
mod deadpool_redis;
