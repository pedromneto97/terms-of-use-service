use std::path::Path;

use crate::domain::errors::TermsOfUseError;

pub trait StorageService: Send + Sync {
    async fn upload_file(&self, file: &Path, content_type: &str)
    -> Result<String, TermsOfUseError>;

    async fn delete_file(&self, path: &str) -> Result<(), TermsOfUseError>;

    async fn get_file_url(&self, path: &str) -> Result<String, TermsOfUseError>;
}
