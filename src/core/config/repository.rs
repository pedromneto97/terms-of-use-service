pub type AppRepository = outbound::DynDatabaseRepository;

pub async fn build_repository() -> AppRepository {
    outbound::build_repository().await
}
