use outbound::{AppRepository, Cache, Publisher, Storage};

#[derive(Clone, Debug)]
pub struct Config {
    pub repository: AppRepository,
    pub cache: Cache,
    pub storage: Storage,
    pub publisher: Publisher,
}

impl Config {
    pub async fn new() -> Self {
        let repository = AppRepository::new().await;
        let cache = Cache::new().await;
        let storage = Storage::new().await;
        let publisher = Publisher::new().await;

        Config {
            repository,
            cache,
            storage,
            publisher,
        }
    }
}
