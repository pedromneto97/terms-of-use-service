use std::path::PathBuf;

use tokio::fs::File;
use tonic::Status;
use tracing::error;
use uuid::Uuid;

pub async fn create_temp_file() -> Result<(File, PathBuf), Status> {
    let file_name = Uuid::new_v4().to_string();
    let file_path = std::env::temp_dir().join(&file_name);

    let file = File::create(&file_path).await.map_err(|e| {
        error!("Failed to create temp file for term upload: {e}");

        Status::internal(format!("Failed to create temp file for term upload: {e}"))
    })?;

    Ok((file, file_path))
}

#[cfg(test)]
mod tests {
    use super::create_temp_file;

    #[tokio::test]
    async fn test_create_temp_file_success() {
        let (file, path) = create_temp_file().await.unwrap();

        assert!(path.exists());
        assert!(path.is_file());

        let temp_dir = std::env::temp_dir();
        assert!(path.starts_with(temp_dir));

        drop(file);
        let _ = tokio::fs::remove_file(&path).await;
    }
}
