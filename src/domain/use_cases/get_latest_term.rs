use crate::domain::{
    data::{repository::TermRepository, service::CacheService},
    entities::TermOfUse,
    errors::TermsOfUseError,
};

pub async fn get_latest_term_use_case(
    repository: &impl TermRepository,
    cache_service: &impl CacheService,
    group: &str,
) -> Result<TermOfUse, TermsOfUseError> {
    let term = cache_service.get_latest_term_for_group(group).await;
    if let Ok(Some(term)) = term {
        return Ok(term);
    }

    let term = repository
        .get_latest_term_for_group(group)
        .await?
        .ok_or(TermsOfUseError::NotFound)?;

    let _ = cache_service.store_latest_term_for_group(&term).await;

    Ok(term)
}
