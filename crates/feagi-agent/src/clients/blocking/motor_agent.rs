use crate::FeagiAgentError;
use feagi_io::traits_and_enums::client::{
    FeagiClientSubscriber, FeagiClientSubscriberProperties,
};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;

pub struct MotorAgent {
    properties: Box<dyn FeagiClientSubscriberProperties>,
    subscriber: Option<Box<dyn FeagiClientSubscriber>>,
    receive_buffer: FeagiByteContainer,
}

impl MotorAgent {
    pub fn new(
        properties: Box<dyn FeagiClientSubscriberProperties>,
        agent_id: AgentID,
    ) -> MotorAgent {
        let mut buffer = FeagiByteContainer::new_empty();
        buffer.set_agent_identifier(agent_id);

        MotorAgent {
            properties,
            subscriber: None,
            receive_buffer:buffer,
        }
    }

    pub fn get_receive_buffer(&mut self) -> &mut FeagiByteContainer {
        &mut self.receive_buffer
    }

    pub fn request_connect(&mut self) -> Result<(), FeagiAgentError> {
        if self.subscriber.is_none() {
            self.subscriber = Some(self.properties.as_boxed_client_subscriber());
        }

        let subscriber = self.subscriber.as_mut().unwrap();

        match subscriber.poll() {
            FeagiEndpointState::Inactive => {
                subscriber.request_connect()?;
                Ok(())
            }
            _ => Err(FeagiAgentError::ConnectionFailed(
                "Socket is already active!".to_string(),
            )),
        }
    }

    pub fn receive_into_buffer(&mut self) -> Result<(), FeagiAgentError> {
        let subscriber = self.subscriber.as_mut().ok_or_else(|| {
            FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string())
        })?;

        let state_snapshot = subscriber.poll().clone();
        match state_snapshot {
            FeagiEndpointState::Inactive => Err(FeagiAgentError::UnableToDecodeReceivedData(
                "Cannot receive from inactive socket".to_string(),
            )),
            FeagiEndpointState::Pending => Err(FeagiAgentError::UnableToDecodeReceivedData(
                "Cannot receive from pending socket".to_string(),
            )),
            FeagiEndpointState::ActiveWaiting => Err(FeagiAgentError::UnableToDecodeReceivedData(
                "No motor data available".to_string(),
            )),
            FeagiEndpointState::ActiveHasData => {
                let data = subscriber.consume_retrieved_data()?;
                self.receive_buffer
                    .try_write_data_by_copy_and_verify(data)?;
                Ok(())
            }
            FeagiEndpointState::Errored(err) => {
                subscriber.confirm_error_and_close()?;
                Err(FeagiAgentError::ConnectionFailed(err.to_string()))
            }
        }
    }

    pub fn poll_for_motor_data(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        let subscriber = self.subscriber.as_mut().ok_or_else(|| {
            FeagiAgentError::ConnectionFailed("No socket is active to poll!".to_string())
        })?;

        let state = subscriber.poll().clone();
        match state {
            FeagiEndpointState::Inactive => {Ok(None)}
            FeagiEndpointState::Pending => {Ok(None)}
            FeagiEndpointState::ActiveWaiting => {
                // return data
                let data = subscriber.consume_retrieved_data()?;
                self.receive_buffer.try_write_data_by_copy_and_verify(data)?;
                Ok(Some(&self.receive_buffer))
            }
            FeagiEndpointState::ActiveHasData => {
                // Not Possible
                return Err(FeagiAgentError::UnableToSendData("Sensor Socket has recieved data!".to_string()));
            }
            FeagiEndpointState::Errored(_) => {
                subscriber.confirm_error_and_close()?;
                return Err(FeagiAgentError::ConnectionFailed("Connection failed".to_string()));
            }
        }
    }
}
