use crate::{
    data::{repository::DatabaseRepository, service::CacheService},
    errors::{Result, TermsOfUseError},
};

#[tracing::instrument(skip(repository, cache, user_id, group))]
pub async fn has_user_agreed_to_term_use_case(
    repository: &dyn DatabaseRepository,
    cache: &dyn CacheService,
    user_id: i32,
    group: &str,
) -> Result<bool> {
    if let Some(agreed) = cache
        .find_user_agreement(user_id, group)
        .await
        .unwrap_or(None)
    {
        return Ok(agreed);
    }

    let latest_term = repository
        .get_latest_term_for_group(group)
        .await?
        .ok_or(TermsOfUseError::NotFound)?;

    let agreed = repository
        .has_user_agreed_to_term(user_id, latest_term.id)
        .await?;

    let _ = cache.store_user_agreement(user_id, group, agreed).await;

    Ok(agreed)
}
