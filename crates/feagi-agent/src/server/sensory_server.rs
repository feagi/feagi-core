use feagi_io::core::::server::{FeagiServerPublisher, FeagiServerPuller};
use feagi_serialization::FeagiByteContainer;
use crate::FeagiAgentClientError;

pub struct SensoryServer {
    puller: Box<dyn FeagiServerPuller>
}

impl SensoryServer {
    pub async fn new(puller: Box<dyn FeagiServerPuller>) -> Result<Self, FeagiAgentClientError> {
        let mut server = Self { puller };
        let result = server.puller.start().await;
        match result {
            Ok(_) => Ok(server),
            Err(e) => Err(FeagiAgentClientError::GeneralFailure(format!("{}", e))),
        }
    }

    pub async fn poll_for_sensor_data(&mut self, sensory_neuron_data_write_target: &mut FeagiByteContainer) -> Result<(), FeagiAgentClientError> {
        let result = self.puller.try_poll_receive().await
            .map_err(|_| FeagiAgentClientError::GeneralFailure("Failed to poll sensor endpoint!".to_string()))?;
        // TODO there has to be a better way to do this without a copy
        sensory_neuron_data_write_target.try_write_data_by_ownership_to_container_and_verify(result)
            .map_err(|_| FeagiAgentClientError::GeneralFailure("Incoming data did not deserialize into neuron data!".to_string()))?;
        Ok(())
    }
}