use crate::core::config::{cache::Cache, repository::AppRepository, storage::Storage};

mod cache;
mod repository;
mod storage;

#[derive(Clone, Debug)]
pub struct Config {
    pub repository: AppRepository,
    pub cache: Cache,
    pub storage: Storage,
}

impl Config {
    pub async fn new() -> Self {
        let repository = AppRepository::new().await;
        let cache = Cache::new().await;
        let storage = Storage::new().await;

        Config {
            repository,
            cache,
            storage,
        }
    }
}
