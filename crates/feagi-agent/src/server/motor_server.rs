use feagi_io::traits_and_enums::server::FeagiServerPublisher;
use feagi_serialization::FeagiByteContainer;
use crate::FeagiAgentError;

/// Publishes motor data
pub struct MotorServer {
    publisher: Box<dyn FeagiServerPublisher>
}

impl MotorServer {
    pub async fn new(publisher: Box<dyn FeagiServerPublisher>) -> Result<Self, FeagiAgentError> {
        let mut server = Self { publisher };
        let result = server.publisher.start().await;
        match result {
            Ok(_) => Ok(server),
            Err(e) => Err(FeagiAgentError::GeneralFailure(format!("{}", e))),
        }
    }

    pub async fn poll(&mut self) -> Result<(), FeagiAgentError> {
        self.publisher.poll().await
            .map_err(|_| FeagiAgentError::GeneralFailure("Failed to poll motor endpoint!".to_string()))
    }

    pub async fn publish(&mut self, neuron_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        self.publisher.publish(neuron_data.get_byte_ref()).await.map_err(
            |e| FeagiAgentError::ServerFailedToSendData(e.to_string()))
    }
}