mod core;
pub(crate) mod domain;
mod inbound;
mod outbound;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = core::Config::new().await;

    inbound::start_server(config).await
}
