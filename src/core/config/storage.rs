pub type Storage = outbound::DynStorage;

pub async fn build_storage() -> Storage {
    outbound::build_storage().await
}
