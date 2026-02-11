use feagi_io::traits_and_enums::server::FeagiServerPuller;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::{FeagiByteContainer, SessionID};

use crate::FeagiAgentError;

/// Handles inbound sensor stream polling and byte-cache updates for one session.
pub struct SensorTranslator {
    sensor_server: Box<dyn FeagiServerPuller>,
    sensor_byte_cache: FeagiByteContainer,
}

impl SensorTranslator {
    pub fn new(session_id: SessionID, sensor_server: Box<dyn FeagiServerPuller>) -> Self {
        let mut sensor_byte_cache = FeagiByteContainer::new_empty();
        let _ = sensor_byte_cache.set_session_id(session_id);
        Self {
            sensor_server,
            sensor_byte_cache,
        }
    }

    pub fn poll_sensor_server(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        let sensor_server = &mut self.sensor_server;
        match sensor_server.poll().clone() {
            FeagiEndpointState::Inactive
            | FeagiEndpointState::Pending
            | FeagiEndpointState::ActiveWaiting => Ok(None),
            FeagiEndpointState::ActiveHasData => {
                let data = sensor_server.consume_retrieved_data()?;
                self.sensor_byte_cache.try_write_data_by_copy_and_verify(data)?;
                Ok(Some(&self.sensor_byte_cache))
            }
            FeagiEndpointState::Errored(error) => {
                sensor_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }
}
