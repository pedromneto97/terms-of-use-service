use crate::core::config::{cache::Cache, repository::AppRepository};

mod cache;
mod repository;

pub struct Config {
    pub repository: AppRepository,
    pub cache: Cache,
}
