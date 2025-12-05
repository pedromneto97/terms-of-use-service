use std::path::Path;

use chrono::Utc;

use crate::domain::{
    data::{
        repository::TermRepository,
        service::{CacheService, StorageService},
    },
    dto::CreateTermOfUseDTO,
    entities::TermOfUse,
    errors::TermsOfUseError,
};

pub async fn create_term_of_use_use_case(
    repository: &impl TermRepository,
    upload_service: &impl StorageService,
    cache_service: &impl CacheService,
    term: CreateTermOfUseDTO,
    file_path: &Path,
) -> Result<TermOfUse, TermsOfUseError> {
    let latest_term = repository.get_latest_term_for_group(&term.group).await?;
    let next_version = match latest_term {
        Some(t) => t.version + 1,
        None => 1,
    };

    let uploaded_file = upload_service.upload_file(file_path).await?;

    let new_term = TermOfUse {
        id: 0,
        group: term.group,
        version: next_version,
        url: uploaded_file.clone(),
        created_at: Utc::now().naive_utc(),
        info: term.info,
    };

    match repository.create_term(new_term).await {
        Ok(mut created_term) => {
            let _ = cache_service
                .invalidate_cache_for_group(&created_term.group)
                .await;

            created_term.url = upload_service.get_file_url(&created_term.url).await?;

            Ok(created_term)
        }
        Err(e) => {
            let _ = upload_service.delete_file(&uploaded_file).await;

            Err(e)
        }
    }
}
