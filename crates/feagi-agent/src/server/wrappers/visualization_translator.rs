use crate::FeagiAgentError;
use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;

// TODO Error handling, error states if one stream fails

pub struct VisualizationTranslator {
    session_id: AgentID,
    visualization_server: Box<dyn FeagiServerPublisher>,
}

impl VisualizationTranslator {
    pub fn new(session_id: AgentID, visualization_server: Box<dyn FeagiServerPublisher>) -> Self {
        VisualizationTranslator {
            session_id,
            visualization_server,
        }
    }

    #[allow(dead_code)]
    pub fn get_session_id(&self) -> AgentID {
        self.session_id
    }

    /// Consume this translator and return reusable publisher properties.
    ///
    /// This is used during deregistration to recycle the endpoint back into
    /// the available transport pool.
    pub fn into_publisher_properties(self) -> Box<dyn FeagiServerPublisherProperties> {
        self.visualization_server.as_boxed_publisher_properties()
    }

    /// Poll visualization server to keep it alive
    pub fn poll_visualization_server(&mut self) -> Result<(), FeagiAgentError> {
        let viz_server = &mut self.visualization_server;
        let state = viz_server.poll().clone();
        match state {
            FeagiEndpointState::Inactive => Ok(()),
            FeagiEndpointState::Pending => Ok(()),
            FeagiEndpointState::ActiveWaiting => Ok(()),
            FeagiEndpointState::ActiveHasData => Err(FeagiAgentError::SocketFailure(
                "Agent cannot send Visualization data!".to_string(),
            )),
            FeagiEndpointState::Errored(error) => {
                self.visualization_server.confirm_error_and_close()?;
                Err(FeagiAgentError::SocketFailure(error.to_string()))
            }
        }
    }

    /// Send visualization data over the dedicated visualization socket
    pub fn poll_and_send_visualization_data(
        &mut self,
        viz_data: &FeagiByteContainer,
    ) -> Result<(), FeagiAgentError> {
        let viz_server = &mut self.visualization_server;
        let state = viz_server.poll();
        match state {
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
