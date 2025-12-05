use crate::core::Config;

#[cfg(feature = "actix-web")]
mod actix;

pub async fn start_server(config: Config) -> std::io::Result<()> {
    #[cfg(feature = "actix-web")]
    actix::start_server(config).await
}
