#[cfg(feature = "sns")]
pub type Publisher = crate::outbound::publisher::sns::SNSPublisher;

#[cfg(not(feature = "publisher"))]
pub type Publisher = crate::outbound::publisher::noop::NoopPublisher;
