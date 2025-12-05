#[cfg(feature = "redis")]
pub type Cache = crate::outbound::cache::redis::RedisConfig;

#[cfg(not(feature = "cache"))]
pub type Cache = crate::outbound::cache::noop::NoopCacheService;
