#[cfg(any(not(feature = "publisher"), clippy, rustfmt, test))]
pub mod noop;

#[cfg(feature = "sns")]
pub mod sns;

#[cfg(feature = "kafka")]
pub mod kafka;
