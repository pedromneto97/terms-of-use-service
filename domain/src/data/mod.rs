use crate::data::{
    health_check::HealthCheck,
    repository::DatabaseRepository,
    service::{CacheService, PublisherService, StorageService},
};

pub mod health_check;
pub mod repository;
pub mod service;

pub trait DatabaseRepositoryWithHealthCheck:
    DatabaseRepository + HealthCheck + Send + Sync
{
}

pub trait CacheServiceWithHealthCheck: CacheService + HealthCheck + Send + Sync {}

pub trait PublisherServiceWithHealthCheck: PublisherService + HealthCheck + Send + Sync {}

pub trait StorageServiceWithHealthCheck: StorageService + HealthCheck + Send + Sync {}
