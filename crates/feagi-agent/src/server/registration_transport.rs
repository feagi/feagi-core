//! Transport adapter interface for agent registration.
//!
//! Allows REST, ZMQ, WebSocket, and future transports to plug into the same
//! core registration path without changing handler logic.

use feagi_io::core::traits_and_enums::server::FeagiServerRouter;
use feagi_serialization::SessionID;

use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::registration::{RegistrationRequest, RegistrationResponse};

/// Poll-based registration source: transports that deliver registration requests
/// asynchronously (e.g. ZMQ, WebSocket). The handler polls each source and, when
/// a request is ready, calls the core and sends the response back via this trait.
pub trait PollableRegistrationSource: Send {
    /// Advance transport state. Returns a pending registration if one is ready.
    /// `None` means no request ready (Inactive, Pending, ActiveWaiting, or already consumed).
    fn poll_registration(
        &mut self,
    ) -> Result<Option<(SessionID, RegistrationRequest)>, FeagiAgentServerError>;

    /// Send the registration response back to the client identified by `session_id`.
    fn send_response(
        &mut self,
        session_id: SessionID,
        response: &RegistrationResponse,
    ) -> Result<(), FeagiAgentServerError>;

    /// Human-readable name for logging and diagnostics.
    fn source_name(&self) -> &str {
        "registration-source"
    }
}

/// Wraps a byte-level [`FeagiServerRouter`] and implements [`PollableRegistrationSource`]
/// using JSON (de)serialization. Used so the handler only deals with
/// `RegistrationRequest`/`RegistrationResponse` and does not depend on raw router bytes.
pub struct RouterRegistrationAdapter {
    router: Box<dyn FeagiServerRouter>,
    request_buffer: Vec<u8>,
    source_name: String,
}

impl RouterRegistrationAdapter {
    /// Build an adapter from a boxed router. The router must already be started
    /// (e.g. `request_start()` called and polled to ActiveWaiting) by the caller.
    pub fn new(router: Box<dyn FeagiServerRouter>, source_name: impl Into<String>) -> Self {
        Self {
            router,
            request_buffer: Vec::new(),
            source_name: source_name.into(),
        }
    }
}

impl PollableRegistrationSource for RouterRegistrationAdapter {
    fn poll_registration(
        &mut self,
    ) -> Result<Option<(SessionID, RegistrationRequest)>, FeagiAgentServerError> {
        use feagi_io::core::traits_and_enums::FeagiEndpointState;

        let state = self.router.poll();
        match state {
            FeagiEndpointState::ActiveHasData => {
                let (session_id, bytes) = self.router.consume_retrieved_request().map_err(|e| {
                    FeagiAgentServerError::UnableToDecodeReceivedData(e.to_string())
                })?;
                self.request_buffer.clear();
                self.request_buffer.extend_from_slice(bytes);
                let request: RegistrationRequest = serde_json::from_slice(&self.request_buffer)
                    .map_err(|e| {
                        FeagiAgentServerError::UnableToDecodeReceivedData(format!(
                            "Failed to parse RegistrationRequest: {}",
                            e
                        ))
                    })?;
                Ok(Some((session_id, request)))
            }
            FeagiEndpointState::Errored(_) => {
                self.router
                    .confirm_error_and_close()
                    .map_err(|err| FeagiAgentServerError::ConnectionFailed(err.to_string()))?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn send_response(
        &mut self,
        session_id: SessionID,
        response: &RegistrationResponse,
    ) -> Result<(), FeagiAgentServerError> {
        let bytes = serde_json::to_vec(response)
            .map_err(|e| FeagiAgentServerError::UnableToSendData(e.to_string()))?;
        self.router
            .publish_response(session_id, &bytes)
            .map_err(|e| FeagiAgentServerError::UnableToSendData(e.to_string()))?;
        Ok(())
    }

    fn source_name(&self) -> &str {
        &self.source_name
    }
}
