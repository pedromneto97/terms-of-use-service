use std::path::Path;

use crate::errors::Result;

#[cfg_attr(test, mockall::automock)]
pub trait StorageService: Send + Sync {
    async fn upload_file(&self, file: &Path, content_type: &str) -> Result<String>;

    async fn delete_file(&self, path: &str) -> Result<()>;

    async fn get_file_url(&self, path: &str) -> Result<String>;
}
