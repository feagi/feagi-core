use thiserror::Error;

/// Rejected Zenoh configuration.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ZenohConfigBuildError {
    /// `ZenohTimeoutMs::Millis(0)` duplicates `NoRetry`.
    #[error("a timeout of 0 milliseconds is ZenohTimeoutMs::NoRetry")]
    ZeroMillisecondTimeout,
    /// An endpoint string is missing its `<proto>/<address>` form.
    #[error("endpoint `{locator}` must use the form <proto>/<address>")]
    InvalidEndpoint { locator: String },
    /// An endpoint group listed no locators.
    #[error("an endpoint group must contain at least one locator")]
    EmptyEndpointGroup,
    /// Namespace failed key-expression checks.
    #[error("namespace `{prefix}` is invalid: {reason}")]
    InvalidNamespace { prefix: String, reason: &'static str },
    /// LowLatency was enabled while unicast QoS was still enabled.
    #[error("unicast lowlatency requires qos to be disabled")]
    LowLatencyWithQos,
    /// A priority queue size was outside 1 through 16.
    #[error("tx queue `{queue}` size is {value}; Zenoh accepts 1 through 16")]
    QueueSize { queue: &'static str, value: u8 },
    /// `transport.link.protocols` was an empty list.
    #[error("the link protocol list is empty; leave protocols unset to allow every supported protocol")]
    EmptyProtocolList,
    /// `batch_size` was 0.
    #[error("link tx batch_size must be at least 1 byte")]
    ZeroBatchSize,
    /// `rx.buffer_size` was 0.
    #[error("link rx buffer_size must be at least 1 byte")]
    ZeroReceiveBuffer,
    /// `rx.max_message_size` was 0.
    #[error("link rx max_message_size must be at least 1 byte")]
    ZeroMaxMessageSize,
    /// The receiver cannot hold one transmit batch.
    #[error("link rx max_message_size must be at least the tx batch_size")]
    MaxMessageSmallerThanBatch,
    /// Shared-memory optimization was enabled with a zero pool.
    #[error("shared-memory transport optimization requires a non-zero pool_size")]
    EmptySharedMemoryPool,
    /// A password was set without a username.
    #[error("a transport auth password requires a username")]
    PasswordWithoutUser,
    /// A retry period was negative.
    #[error("a retry period must be zero or positive")]
    NegativeRetryPeriod,
    /// A retry increase factor was not a finite number greater than zero.
    #[error("the retry increase factor must be finite and greater than 0")]
    InvalidRetryFactor,
    /// `period_max_ms` was below `period_init_ms`.
    #[error("retry period_max_ms must be greater than or equal to period_init_ms")]
    RetryMaxBelowInit,
    /// `serde_json` could not write the document.
    #[error("failed to encode JSON5: {0}")]
    Json5(String),
}
