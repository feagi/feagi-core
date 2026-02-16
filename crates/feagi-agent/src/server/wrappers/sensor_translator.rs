use crate::FeagiAgentError;
use feagi_io::traits_and_enums::server::{FeagiServerPuller, FeagiServerPullerProperties};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;
use tracing::debug;

// TODO Error handling, error states if one stream fails

pub struct SensorTranslator {
    session_id: AgentID,
    sensor_server: Box<dyn FeagiServerPuller>,
    sensor_byte_cache: FeagiByteContainer,
}

impl SensorTranslator {
    fn should_drop_malformed_sensor_frame(err: &FeagiAgentError) -> bool {
        matches!(
            err,
            FeagiAgentError::UnableToDecodeReceivedData(msg)
                if msg.contains("Given Feagi Byte Structure byte length is too short")
        )
    }

    pub fn new(session_id: AgentID, sensor_server: Box<dyn FeagiServerPuller>) -> Self {
        let mut sensor_byte_cache = FeagiByteContainer::new_empty();
        let _ = sensor_byte_cache.set_agent_identifier(session_id);

        SensorTranslator {
            session_id,
            sensor_server,
            sensor_byte_cache,
        }
    }

    #[allow(dead_code)]
    pub fn get_session_id(&self) -> AgentID {
        self.session_id
    }

    /// Consume this translator and return reusable puller properties.
    ///
    /// This is used during deregistration to recycle the endpoint back into
    /// the available transport pool.
    pub fn into_puller_properties(self) -> Box<dyn FeagiServerPullerProperties> {
        self.sensor_server.as_boxed_puller_properties()
    }

    /// Poll the sensor server, getting any incoming byte data if there is new
    pub fn poll_sensor_server(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        let sensor_server = &mut self.sensor_server;
        let state = sensor_server.poll().clone();
        match state {
            FeagiEndpointState::Inactive => Ok(None),
            FeagiEndpointState::Pending => Ok(None),
            FeagiEndpointState::ActiveWaiting => Ok(None),
            FeagiEndpointState::ActiveHasData => {
                let data = self.sensor_server.consume_retrieved_data()?;
                match self.sensor_byte_cache.try_write_data_by_copy_and_verify(data) {
                    Ok(()) => Ok(Some(&self.sensor_byte_cache)),
                    Err(e) => {
                        let agent_err: FeagiAgentError = e.into();
                        if Self::should_drop_malformed_sensor_frame(&agent_err) {
                            debug!(
                                "Dropping malformed short sensor frame for session {}",
                                self.session_id.to_base64()
                            );
                            Ok(None)
                        } else {
                            Err(agent_err)
                        }
                    }
                }
            }
            FeagiEndpointState::Errored(error) => {
                sensor_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }
}
