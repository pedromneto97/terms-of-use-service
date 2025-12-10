#[cfg(feature = "sns")]
pub type Publisher = crate::outbound::publisher::sns::SNSPublisher;

#[cfg(feature = "kafka")]
pub type Publisher = crate::outbound::publisher::kafka::KafkaPublisher;

#[cfg(not(feature = "publisher"))]
pub type Publisher = crate::outbound::publisher::noop::NoopPublisher;
