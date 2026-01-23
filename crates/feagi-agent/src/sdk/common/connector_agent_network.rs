use feagi_io::io_api::FeagiNetworkError;
use feagi_io::io_api::implementations::zmq::{FEAGIZMQClientPusher, FEAGIZMQClientSubscriber};
use feagi_io::io_api::traits_and_enums::client::{FeagiClientPusher, FeagiClientSubscriber};

/// Marker trait for connector agent network implementations.
pub trait ConnectorAgentNetworkImplementation {
}

pub struct ConnectorAgentNetwork {
    sensor_stream: Box<dyn FeagiClientPusher>,
    motor_stream: Box<dyn FeagiClientSubscriber>,
}

impl ConnectorAgentNetwork {
    /// Create a ZMQ-based connector network with sensor/motor streams.
    pub fn new_zmq(
        sensor_endpoint: String,
        motor_endpoint: String,
    ) -> Result<ConnectorAgentNetwork, FeagiNetworkError> {
        let sensor_stream = FEAGIZMQClientPusher::new(sensor_endpoint, Box::new(|_change| {}))?;
        let motor_stream = FEAGIZMQClientSubscriber::new(motor_endpoint, Box::new(|_change| {}))?;
        Ok(ConnectorAgentNetwork {
            sensor_stream: Box::new(sensor_stream),
            motor_stream: Box::new(motor_stream),
        })
    }
}