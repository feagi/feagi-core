use feagi_io::traits_and_enums::server::FeagiServerPublisher;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::FeagiByteContainer;
use crate::agent_id::AgentID;
use crate::FeagiAgentError;

// TODO Error handling, error states if one stream fails

pub struct MotorTranslator {
    session_id: AgentID,
    motor_server: Box<dyn FeagiServerPublisher>,
}

impl MotorTranslator {

    pub fn new(
        session_id: AgentID,
        motor_server: Box<dyn FeagiServerPublisher>,
    ) -> Self {
        MotorTranslator {
            session_id,
            motor_server,
        }
    }

    pub fn get_session_id(&self) -> AgentID {
        self.session_id
    }

    /// Poll motor server to keep it alive
    pub fn poll_motor_server(&mut self) -> Result<(), FeagiAgentError> {
        let motor_server = &mut self.motor_server;
        let state = motor_server.poll().clone();
        match state {
            FeagiEndpointState::Inactive => {
                Ok(())
            }
            FeagiEndpointState::Pending => {
                Ok(())
            }
            FeagiEndpointState::ActiveWaiting => {
                Ok(())
            }
            FeagiEndpointState::ActiveHasData => {
                // Not possible, a motor should never send data!
                // TODO proper way to close this socket
                Err(FeagiAgentError::SocketFailure("Agent cannot send Motor data!".to_string()))
            }
            FeagiEndpointState::Errored(error) => {
                self.motor_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }

    /// Send motor byte data
    pub fn poll_and_send_buffered_motor_data(&mut self, motor_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let motor_server = &mut self.motor_server;
        let state = motor_server.poll();
        match state {
            FeagiEndpointState::ActiveWaiting => {
                motor_server.publish_data(motor_data.get_byte_ref())?;
                Ok(())
            }
            _ => {
                // Socket is not in a state to handle incoming data
                Err(FeagiAgentError::UnableToSendData("Socket is not in a state to send data!".to_string()))
            }
        }
    }

}