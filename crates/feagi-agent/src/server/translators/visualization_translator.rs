use feagi_io::traits_and_enums::server::FeagiServerPublisher;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::FeagiByteContainer;

use crate::FeagiAgentError;

/// Handles outbound visualization stream lifecycle and publishing for one session.
pub struct VisualizationTranslator {
    visualization_server: Box<dyn FeagiServerPublisher>,
}

impl VisualizationTranslator {
    pub fn new(visualization_server: Box<dyn FeagiServerPublisher>) -> Self {
        Self {
            visualization_server,
        }
    }

    pub fn poll_visualization_server(&mut self) -> Result<(), FeagiAgentError> {
        let viz_server = &mut self.visualization_server;
        match viz_server.poll().clone() {
            FeagiEndpointState::Inactive
            | FeagiEndpointState::Pending
            | FeagiEndpointState::ActiveWaiting => Ok(()),
            FeagiEndpointState::ActiveHasData => Err(FeagiAgentError::SocketFailure(
                "Agent cannot send Visualization data!".to_string(),
            )),
            FeagiEndpointState::Errored(error) => {
                viz_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }

    pub fn send_visualization_data(
        &mut self,
        viz_data: &FeagiByteContainer,
    ) -> Result<(), FeagiAgentError> {
        let viz_server = &mut self.visualization_server;
        match viz_server.poll() {
            FeagiEndpointState::ActiveWaiting => {
                viz_server.publish_data(viz_data.get_byte_ref())?;
                Ok(())
            }
            _ => Err(FeagiAgentError::UnableToSendData(
                "Visualization socket not ready!".to_string(),
            )),
        }
    }
}
