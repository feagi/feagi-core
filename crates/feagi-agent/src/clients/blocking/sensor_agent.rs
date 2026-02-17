use crate::FeagiAgentError;
use feagi_io::traits_and_enums::client::{FeagiClientPusher, FeagiClientPusherProperties};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;

pub struct SensorAgent {
    properties: Box<dyn FeagiClientPusherProperties>,
    pusher: Option<Box<dyn FeagiClientPusher>>,
}

impl SensorAgent {
    pub fn new(properties: Box<dyn FeagiClientPusherProperties>, agent_id: AgentID) -> SensorAgent {
        let mut buffer = FeagiByteContainer::new_empty();
        let _ = buffer.set_agent_identifier(agent_id);

        SensorAgent {
            properties,
            pusher: None,
        }
    }

    pub fn request_connect(&mut self) -> Result<(), FeagiAgentError> {
        if self.pusher.is_none() {
            self.pusher = Some(self.properties.as_boxed_client_pusher());
        }

        let pusher = self.pusher.as_mut().unwrap();

        match pusher.poll() {
            FeagiEndpointState::Inactive => {
                _ = pusher.request_connect()?;
                Ok(())
            }
            _ => Err(FeagiAgentError::ConnectionFailed(
                "Socket is already active!".to_string(),
            )),
        }
    }

    pub fn send_buffer(&mut self, buffer: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        let pusher = self.pusher.as_mut().ok_or_else(|| {
            FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string())
        })?;

        let state_snapshot = pusher.poll().clone();
        match state_snapshot {
            FeagiEndpointState::Inactive => {
                return Err(FeagiAgentError::UnableToSendData(
                    "Cannot send to inactive socket".to_string(),
                ));
            }
            FeagiEndpointState::Pending => {
                return Err(FeagiAgentError::UnableToSendData(
                    "Cannot send to pending socket".to_string(),
                ));
            }
            FeagiEndpointState::ActiveWaiting => {
                pusher.publish_data(buffer.get_byte_ref())?;
                Ok(())
            }
            FeagiEndpointState::ActiveHasData => {
                // Impossible for sensor
                return Err(FeagiAgentError::UnableToSendData(
                    "Socket has data!".to_string(),
                ));
            }
            FeagiEndpointState::Errored(err) => {
                pusher.confirm_error_and_close()?;
                return Err(FeagiAgentError::ConnectionFailed(err.to_string()));
            }
        }
    }

    pub fn poll(&mut self) -> Result<(), FeagiAgentError> {
        let pusher = self.pusher.as_mut().ok_or_else(|| {
            FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string())
        })?;

        let state = pusher.poll().clone();
        match state {
            FeagiEndpointState::Inactive => Ok(()),
            FeagiEndpointState::Pending => Ok(()),
            FeagiEndpointState::ActiveWaiting => {
                // Do nothing
                Ok(())
            }
            FeagiEndpointState::ActiveHasData => {
                // Not Possible
                return Err(FeagiAgentError::UnableToSendData(
                    "Sensor Socket has recieved data!".to_string(),
                ));
            }
            FeagiEndpointState::Errored(_) => {
                pusher.confirm_error_and_close()?;
                return Err(FeagiAgentError::ConnectionFailed(
                    "Connection failed".to_string(),
                ));
            }
        }
    }
}
