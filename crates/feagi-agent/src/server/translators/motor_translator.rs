use feagi_io::traits_and_enums::server::FeagiServerPublisher;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::FeagiByteContainer;

use crate::FeagiAgentError;

/// Handles outbound motor stream lifecycle and publishing for one session.
pub struct MotorTranslator {
    motor_server: Box<dyn FeagiServerPublisher>,
}

impl MotorTranslator {
    pub fn new(motor_server: Box<dyn FeagiServerPublisher>) -> Self {
        Self { motor_server }
    }

    pub fn poll_motor_server(&mut self) -> Result<(), FeagiAgentError> {
        let motor_server = &mut self.motor_server;
        match motor_server.poll().clone() {
            FeagiEndpointState::Inactive
            | FeagiEndpointState::Pending
            | FeagiEndpointState::ActiveWaiting => Ok(()),
            FeagiEndpointState::ActiveHasData => Err(FeagiAgentError::SocketFailure(
                "Agent cannot send Motor data!".to_string(),
            )),
            FeagiEndpointState::Errored(error) => {
                motor_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }

    pub fn send_buffered_motor_data(
        &mut self,
        motor_data: &FeagiByteContainer,
    ) -> Result<(), FeagiAgentError> {
        let motor_server = &mut self.motor_server;
        match motor_server.poll() {
            FeagiEndpointState::ActiveWaiting => {
                motor_server.publish_data(motor_data.get_byte_ref())?;
                Ok(())
            }
            _ => Err(FeagiAgentError::UnableToSendData(
                "Socket is not in a state to send data!".to_string(),
            )),
        }
    }
}
