//! Shared types for FEAGI networking traits.

use crate::FeagiNetworkError;

/// Represents the current state of a FEAGI network endpoint (client or server).
///
/// This enum is returned by [`super::client::FeagiClient::poll`] and
/// [`super::server::FeagiServer::poll`] to indicate what operations are valid
/// and whether data is available.
///
/// # State Machine
///
/// ```text
/// ┌──────────┐  request_connect/start   ┌─────────┐
/// │ Inactive │ ───────────────────────► │ Pending │
/// └──────────┘                          └────┬────┘
///       ▲                                    │
///       │ request_disconnect/stop            │ (connection established)
///       │                                    ▼
///       │                          ┌─────────────────┐
///       └────────────────────────  │  ActiveWaiting  │ ◄─────┐
///                                  └────────┬────────┘       │
///                                           │                │
///                                           │ (data arrives) │ (data consumed)
///                                           ▼                │
///                                  ┌─────────────────┐       │
///                                  │  ActiveHasData  │ ──────┘
///                                  └─────────────────┘
///
/// Any state can transition to Errored on failure.
/// Call confirm_error_and_close() to return to Inactive from Errored.
/// ```
///
/// # Usage Contract
///
/// - Always call `poll()` before performing send/receive operations
/// - Data operations (`publish_data`, `consume_*`) are only valid in `Active*` states
/// - Sending is valid in the `ActiveWaiting` state
/// - Consuming data is only meaningful in `ActiveHasData`
#[derive(Debug, Clone, PartialEq)]
pub enum FeagiEndpointState {
    /// The endpoint is not connected/bound and not attempting to connect/bind.
    ///
    /// Valid operations: `request_connect()` / `request_start()`
    Inactive,

    /// A connection/bind operation is in progress.
    ///
    /// Continue calling `poll()` until the state transitions to `ActiveWaiting`
    /// or `Errored`. No data operations are valid in this state.
    Pending,

    /// The endpoint is active and ready for data operations, but no incoming
    /// data is currently available.
    ///
    /// Valid operations:
    /// - Sending: `publish_data()`, `publish_request()`, `publish_response()`
    /// - Lifecycle: `request_disconnect()` / `request_stop()`
    ActiveWaiting,

    /// The endpoint is active and has incoming data available to consume.
    ///
    /// This state is only relevant for endpoints that receive data (subscribers,
    /// pullers, requesters awaiting responses, routers with pending requests).
    ///
    /// Valid operations:
    /// - Receiving: `consume_retrieved_data()`, `consume_retrieved_response()`, `consume_retrieved_request()`
    /// - Sending: `publish_data()`, `publish_request()`, `publish_response()`
    /// - Lifecycle: `request_disconnect()` / `request_stop()`
    ActiveHasData,

    /// The endpoint has encountered an error and is no longer operational.
    ///
    /// The contained error describes what went wrong. Call `confirm_error_and_close()`
    /// to acknowledge the error and return to `Inactive` state.
    ///
    /// This state is for persistent errors that require explicit acknowledgment,
    /// not for transient failures that are returned as `Result::Err` from individual
    /// operations.
    Errored(FeagiNetworkError),
}
