use std::path::Path;

use chrono::Utc;

use crate::{
    data::{
        repository::TermRepository,
        service::{CacheService, StorageService},
    },
    dto::CreateTermOfUseDTO,
    entities::TermOfUse,
    errors::Result,
};

#[tracing::instrument(skip(repository, upload_service, cache_service, term, file_path))]
pub async fn create_term_of_use_use_case(
    repository: &dyn TermRepository,
    upload_service: &dyn StorageService,
    cache_service: &dyn CacheService,
    term: CreateTermOfUseDTO,
    file_path: &Path,
    content_type: &str,
) -> Result<TermOfUse> {
    let latest_term = repository.get_latest_term_for_group(&term.group).await?;
    let next_version = match latest_term {
        Some(t) => t.version + 1,
        None => 1,
    };

    let uploaded_file = upload_service.upload_file(file_path, content_type).await?;

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
