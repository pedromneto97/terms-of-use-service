use crate::{
    data::{
        repository::TermRepository,
        service::{CacheService, StorageService},
    },
    entities::TermOfUse,
    errors::{Result, TermsOfUseError},
};

#[tracing::instrument(skip(repository, cache_service, upload_service, group))]
pub async fn get_latest_term_use_case<R: TermRepository, C: CacheService, S: StorageService>(
    repository: &R,
    cache_service: &C,
    upload_service: &S,
    group: &str,
) -> Result<TermOfUse> {
    let term = cache_service.get_latest_term_for_group(group).await;
    if let Ok(Some(term)) = term {
        return Ok(term);
    }

    let mut term = repository
        .get_latest_term_for_group(group)
        .await?
        .ok_or(TermsOfUseError::NotFound)?;

    term.url = upload_service.get_file_url(&term.url).await?;

    let _ = cache_service.store_latest_term_for_group(&term).await;

    Ok(term)
}
