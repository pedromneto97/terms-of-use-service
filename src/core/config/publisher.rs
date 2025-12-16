pub type Publisher = outbound::DynPublisher;

pub async fn build_publisher() -> Publisher {
	outbound::build_publisher().await
}
