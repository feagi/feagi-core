use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPuller};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::{FeagiByteContainer, SessionID};
use crate::FeagiAgentError;

pub struct EmbodimentTranslator {
    session_id: SessionID,
    motor_server: Box<dyn FeagiServerPublisher>,
    sensor_sever: Box<dyn FeagiServerPuller>,
    motor_byte_cache: FeagiByteContainer,
    sensor_byte_cache: FeagiByteContainer,
}

impl EmbodimentTranslator {

    pub fn new(session_id: SessionID, motor_server: Box<dyn FeagiServerPublisher>, sensor_server: Box<dyn FeagiServerPuller>) -> Self {
        let mut motor_byte_cache = FeagiByteContainer::new_empty();
        motor_byte_cache.set_session_id(session_id);
        let mut sensor_byte_cache = FeagiByteContainer::new_empty();
        sensor_byte_cache.set_session_id(session_id);

        EmbodimentTranslator {
            session_id,
            motor_server,
            sensor_sever,
            motor_byte_cache,
            sensor_byte_cache
        }
    }

    pub fn get_session_id(&self) -> SessionID {
        self.session_id
    }

    pub fn poll_sensor_server(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        let sensor_sever = &mut self.sensor_sever;
        let state = sensor_sever.poll();
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
                self.process_incoming_sensor_data()
            }
            FeagiEndpointState::Errored(error) => {
                self.sensor_sever.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }

    /// Get byte struct on which you can write motor neuron to
    pub fn get_motor_data_bytes_ref(&mut self) -> &mut FeagiByteContainer {
        &mut self.motor_byte_cache
    }

    /// Poll motor server to keep it alive
    pub fn poll_motor_server(&mut self) -> Result<(), FeagiAgentError> {
        let motor_server = &mut self.motor_server;
        let state = motor_server.poll();
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
    pub fn send_buffered_motor_data(&mut self) -> Result<(), FeagiAgentError> {
        let motor_server = &mut self.motor_server;
        let state = motor_server.poll();
        match state {
            FeagiEndpointState::ActiveWaiting => {
                motor_server.publish_data(self.motor_byte_cache.get_byte_ref())?;
                Ok(())
            }
            _ => {
                // Socket is not in a state to handle incoming data
                Err(FeagiAgentError::UnableToSendData("Socket is not in a state to send data!".to_string()))
            }
        }


    }

    fn process_incoming_sensor_data(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        let data = self.sensor_sever.consume_retrieved_data()?;
        self.sensor_byte_cache.try_write_data_by_copy_and_verify(data)?;
        Ok(Some(&self.sensor_byte_cache))
    }

}