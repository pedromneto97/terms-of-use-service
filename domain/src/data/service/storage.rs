use std::path::Path;

use async_trait::async_trait;

use crate::errors::Result;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StorageService: Send + Sync {
    async fn upload_file(&self, file: &Path, content_type: &str) -> Result<String>;

    async fn delete_file(&self, path: &str) -> Result<()>;

    async fn get_file_url(&self, path: &str) -> Result<String>;
}
