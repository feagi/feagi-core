use feagi_io::next::FeagiNetworkError;
use feagi_io::next::implementations::zmq::FEAGIZMQClientPusher;
use feagi_io::next::traits::client::{FeagiClientPusher, FeagiClientSubscriber};

pub trait ConnectorAgentNetworkImplementation {
    // contains a sensor stream, motor stream
    fn send sensor
    fn motor_recieved

}





pub struct ConnectorAgentNetwork {

    sensor_stream: Box<dyn FeagiClientPusher>,
    motor_stream: Box<dyn FeagiClientSubscriber>,
}

impl ConnectorAgentNetwork {
    pub fn new_zmq(sensor_endpoint: String, motor_endpoint: String) -> Result<ConnectorAgentNetwork, FeagiNetworkError> {

        let sensor_stream = FEAGIZMQClientPusher::new(sensor_endpoint)

    }
}