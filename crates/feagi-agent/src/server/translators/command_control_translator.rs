//! Transport adapter interface for command and control commands.
//!
//! Allows REST, ZMQ, WebSocket, and future transports to plug into the same
//! core path without changing handler logic.

use crate::command_and_control::agent_registration_message::RegistrationRequest;
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, FeagiAgentError};
use feagi_io::traits_and_enums::server::{FeagiServerRouter, FeagiServerRouterProperties};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::{AgentID, FeagiNetworkError};
use feagi_serialization::FeagiByteContainer;
use std::collections::HashMap;

pub type IsNewSessionId = bool;

/// Translates the byte data from clients into [FeagiMessage] for ease of use upstream
pub struct CommandControlTranslator {
    router: Box<dyn FeagiServerRouter>,
    request_buffer: FeagiByteContainer,
    send_buffer: FeagiByteContainer,
}

impl CommandControlTranslator {
    /// Build an adapter from a boxed router. The router must already be started
    /// (e.g. `request_start()` called and polled to ActiveWaiting) by the caller.
    pub fn new(router: Box<dyn FeagiServerRouter>) -> Self {
        Self {
            router,
            request_buffer: FeagiByteContainer::new_empty(),
            send_buffer: FeagiByteContainer::new_empty(),
        }
    }

    /// Poll for incoming messages, returns one if found, along with the session ID and true if the session id seems to be new
    pub fn poll_for_incoming_messages(
        &mut self,
        known_session_ids: &HashMap<AgentID, (AgentDescriptor, Vec<AgentCapabilities>)>,
    ) -> Result<Option<(AgentID, FeagiMessage, IsNewSessionId)>, FeagiAgentError> {
        let state = self.router.poll().clone();
        match state {
            FeagiEndpointState::Inactive => Ok(None),
            FeagiEndpointState::Pending => Ok(None),
            FeagiEndpointState::ActiveWaiting => Ok(None),
            FeagiEndpointState::ActiveHasData => {
                self.process_incoming_data_into_message(known_session_ids)
            }
            FeagiEndpointState::Errored(error) => {
                match error {
                    FeagiNetworkError::CannotBind(err) => {
                        self.router.confirm_error_and_close()?;
                        Err(FeagiAgentError::SocketFailure(err.clone()))
                    }
                    FeagiNetworkError::CannotUnbind(err) => {
                        self.router.confirm_error_and_close()?;
                        Err(FeagiAgentError::SocketFailure(err.clone()))
                    }
                    FeagiNetworkError::CannotConnect(err) => {
                        // Only occurs if sending a command / response, and the agent dies. No need to close
                        Err(FeagiAgentError::UnableToSendData(err.clone()))
                    }
                    FeagiNetworkError::CannotDisconnect(err) => {
                        self.router.confirm_error_and_close()?;
                        Err(FeagiAgentError::SocketFailure(err.clone()))
                    }
                    FeagiNetworkError::SendFailed(err) => {
                        // Only occurs if sending a command / response, and the agent dies. No need to close
                        Err(FeagiAgentError::UnableToSendData(err.clone()))
                    }
                    FeagiNetworkError::ReceiveFailed(err) => {
                        // Client sent weird data
                        Err(FeagiAgentError::UnableToDecodeReceivedData(err.clone()))
                    }
                    FeagiNetworkError::InvalidSocketProperties(err) => {
                        self.router.confirm_error_and_close()?;
                        Err(FeagiAgentError::SocketFailure(err.clone()))
                    }
                    FeagiNetworkError::SocketCreationFailed(err) => {
                        self.router.confirm_error_and_close()?;
                        Err(FeagiAgentError::SocketFailure(err.clone()))
                    }
                    FeagiNetworkError::GeneralFailure(err) => {
                        self.router.confirm_error_and_close()?;
                        Err(FeagiAgentError::SocketFailure(err.clone()))
                    }
                }
            }
        }
    }

    /// Send a message to a specific connected agent
    pub fn send_message(
        &mut self,
        session_id: AgentID,
        message: FeagiMessage,
        increment_counter: u16,
    ) -> Result<(), FeagiAgentError> {
        let container = &mut self.send_buffer;
        message.serialize_to_byte_container(container, session_id, increment_counter)?;
        self.router
            .publish_response(session_id, container.get_byte_ref())?;
        Ok(())
    }

    pub fn get_running_server_properties(&self) -> Box<dyn FeagiServerRouterProperties> {
        self.router.as_boxed_router_properties()
    }

    /// Tries converting incoming data into a [FeagiMessage]
    fn process_incoming_data_into_message(
        &mut self,
        known_session_ids: &HashMap<AgentID, (AgentDescriptor, Vec<AgentCapabilities>)>,
    ) -> Result<Option<(AgentID, FeagiMessage, IsNewSessionId)>, FeagiAgentError> {
        let (session_id, incoming_data) = self.router.consume_retrieved_request()?;

        let is_new_session = !known_session_ids.contains_key(&session_id);

        if is_new_session {
            // New Agent? Just make sure it isnt spam first
            if incoming_data.len() > RegistrationRequest::MAX_REQUEST_SIZE {
                // We are not allowing unknown people to throw large amounts of data. Ignore
                return Ok(None);
            }
        }

        self.request_buffer
            .try_write_data_by_copy_and_verify(incoming_data)?; // Load in data
        let feagi_message: FeagiMessage = (&self.request_buffer).try_into()?;

        // WARNING: It is possible for an agent to request registration a second time. Be wary!
        Ok(Some((session_id, feagi_message, is_new_session)))
    }
}
