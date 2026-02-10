use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPuller};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::FeagiAgentError;

// TODO Error handling, error states if one stream fails

/// Interface for the data streams from / to an Embodiment agent.
pub struct EmbodimentTranslator {
    session_id: SessionID,
    motor_server: Box<dyn FeagiServerPublisher>,
    sensor_server: Box<dyn FeagiServerPuller>,
    sensor_byte_cache: FeagiByteContainer,
}

impl EmbodimentTranslator {

    pub fn new(session_id: SessionID, motor_server: Box<dyn FeagiServerPublisher>, sensor_server: Box<dyn FeagiServerPuller>) -> Self {
        let mut motor_byte_cache = FeagiByteContainer::new_empty();
        let _ = motor_byte_cache.set_session_id(session_id);
        let mut sensor_byte_cache = FeagiByteContainer::new_empty();
        let _ = sensor_byte_cache.set_session_id(session_id);

        EmbodimentTranslator {
            session_id,
            motor_server,
            sensor_server,
            sensor_byte_cache
        }
    }

    pub fn get_session_id(&self) -> SessionID {
        self.session_id
    }

    /// Poll the sensor server, getting any incoming byte data if there is new
    pub fn poll_sensor_server(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        let sensor_server = &mut self.sensor_server;
        let state = sensor_server.poll().clone();
        match state {
            FeagiEndpointState::Inactive => {
                Ok(None)
            }
            FeagiEndpointState::Pending => {
                Ok(None)
            }
            FeagiEndpointState::ActiveWaiting => {
                Ok(None)
            }
            FeagiEndpointState::ActiveHasData => {
                let data = self.sensor_server.consume_retrieved_data()?;
                self.sensor_byte_cache.try_write_data_by_copy_and_verify(data)?;
                Ok(Some(&self.sensor_byte_cache))
            }
            FeagiEndpointState::Errored(error) => {
                sensor_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
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

    /// Send motor byte data (that is already encoded to the motor byte buffer)
    pub fn send_buffered_motor_data(&mut self, motor_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
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