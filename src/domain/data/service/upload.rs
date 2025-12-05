use std::fs::File;

use crate::domain::{dto::UploadedFileDTO, errors::TermsOfUseError};

pub trait UploadService: Send + Sync {
    async fn upload_file(&self, file: &File) -> Result<UploadedFileDTO, TermsOfUseError>;

    async fn delete_file(&self, path: &str) -> Result<(), TermsOfUseError>;
}
