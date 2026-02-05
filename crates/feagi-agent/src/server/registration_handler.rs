//! Transport adapter interface for agent registration.
//!
//! Allows REST, ZMQ, WebSocket, and future transports to plug into the same
//! core registration path without changing handler logic.

use feagi_io::shared::FeagiEndpointState;
use feagi_io::traits_and_enums::server::FeagiServerRouter;
use feagi_serialization::{FeagiByteContainer, SessionID};

use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::registration::{RegistrationRequest, RegistrationResponse};

/// Translates the byte data from clients into [RegistrationRequest] for ease of use upstream
pub struct RegistrationTranslator {
    router: Box<dyn FeagiServerRouter>,
    request_buffer: FeagiByteContainer,
    source_name: String,
}

impl RegistrationTranslator {
    /// Build an adapter from a boxed router. The router must already be started
    /// (e.g. `request_start()` called and polled to ActiveWaiting) by the caller.
    pub fn new(router: Box<dyn FeagiServerRouter>, source_name: impl Into<String>) -> Self {
        Self {
            router,
            request_buffer: FeagiByteContainer::new_empty(),
            source_name: source_name.into(),
        }
    }

    pub fn poll_registration(
        &mut self,
    ) -> Result<Option<(SessionID, RegistrationRequest)>, FeagiAgentServerError> {

        let state = self.router.poll();
        match state {
            FeagiEndpointState::ActiveHasData => {
                let (session_id, bytes) = self
                    .router
                    .consume_retrieved_request()?;

                if bytes.len() > RegistrationRequest::MAX_REQUEST_SIZE {
                    // Silently ignore these, prevent spam attacks
                    // TODO maybe some logging mechanism would be good?
                    return Ok(None);
                }

                self.request_buffer.try_write_data_by_copy_and_verify(bytes)?;


                let request: RegistrationRequest = (&self.request_buffer).try_into()?;
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

    pub fn send_response(
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

    pub fn source_name(&self) -> &str {
        &self.source_name
    }
}

