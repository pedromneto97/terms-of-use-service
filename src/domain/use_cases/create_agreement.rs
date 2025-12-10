use crate::domain::{
    data::{
        repository::{TermRepository, UserAgreementRepository},
        service::{CacheService, PublisherService},
    },
    dto::AcceptedTermOfUseDTO,
    errors::TermsOfUseError,
};

#[tracing::instrument(skip(repository, cache, publisher, user_id, term_id))]
pub async fn create_user_agreement_use_case<
    R: TermRepository + UserAgreementRepository,
    C: CacheService,
    P: PublisherService,
>(
    repository: &R,
    cache: &C,
    publisher: &P,
    user_id: i32,
    term_id: i32,
) -> Result<(), TermsOfUseError> {
    let term = repository
        .get_term_by_id(term_id)
        .await?
        .ok_or(TermsOfUseError::NotFound)?;

    repository.create_user_agreement(user_id, term_id).await?;

    let _ = cache.store_user_agreement(user_id, &term.group, true).await;

    let _ = publisher
        .publish_agreement(AcceptedTermOfUseDTO {
            term_id,
            user_id,
            group: term.group,
        })
        .await;

    Ok(())
}
