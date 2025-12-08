use std::path::PathBuf;

use tokio::fs::File;
use tonic::Status;
use tracing::error;
use uuid::Uuid;

pub async fn create_temp_file() -> Result<(File, PathBuf), Status> {
    let file_name = Uuid::new_v4().to_string();
    let file_path = std::env::temp_dir().join(&file_name);

    let file = tokio::fs::File::create(&file_path).await.map_err(|e| {
        error!("Failed to create temp file for term upload: {e}");

        Status::internal(format!("Failed to create temp file for term upload: {e}"))
    })?;

    Ok((file, file_path))
}
