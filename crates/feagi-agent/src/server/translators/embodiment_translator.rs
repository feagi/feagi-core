use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPuller};
use feagi_serialization::{FeagiByteContainer, SessionID};

use crate::server::translators::{
    MotorTranslator, SensorTranslator, VisualizationTranslator,
};
use crate::FeagiAgentError;

// TODO Error handling, error states if one stream fails

/// Interface for the data streams from / to an Embodiment agent.
pub struct EmbodimentTranslator {
    session_id: SessionID,
    motor_translator: MotorTranslator,
    sensor_translator: SensorTranslator,
    visualization_translator: VisualizationTranslator,
}

impl EmbodimentTranslator {

    pub fn new(
        session_id: SessionID,
        motor_server: Box<dyn FeagiServerPublisher>,
        sensor_server: Box<dyn FeagiServerPuller>,
        visualization_server: Box<dyn FeagiServerPublisher>,
    ) -> Self {
        EmbodimentTranslator {
            session_id,
            motor_translator: MotorTranslator::new(motor_server),
            sensor_translator: SensorTranslator::new(session_id, sensor_server),
            visualization_translator: VisualizationTranslator::new(visualization_server),
        }
    }

    pub fn get_session_id(&self) -> SessionID {
        self.session_id
    }

    /// Poll the sensor server, getting any incoming byte data if there is new
    pub fn poll_sensor_server(&mut self) -> Result<Option<&FeagiByteContainer>, FeagiAgentError> {
        self.sensor_translator.poll_sensor_server()
    }

    /// Poll motor server to keep it alive
    pub fn poll_motor_server(&mut self) -> Result<(), FeagiAgentError> {
        self.motor_translator.poll_motor_server()
    }

    /// Poll visualization server to keep it alive
    pub fn poll_visualization_server(&mut self) -> Result<(), FeagiAgentError> {
        self.visualization_translator.poll_visualization_server()
    }

    /// Send motor byte data (that is already encoded to the motor byte buffer)
    pub fn send_buffered_motor_data(&mut self, motor_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        self.motor_translator.send_buffered_motor_data(motor_data)
    }

    /// Send visualization data over the dedicated visualization socket
    pub fn send_visualization_data(&mut self, viz_data: &FeagiByteContainer) -> Result<(), FeagiAgentError> {
        self.visualization_translator.send_visualization_data(viz_data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use feagi_io::traits_and_enums::server::{FeagiServer, FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller};
    use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint, TransportProtocolImplementation};
    use feagi_io::FeagiNetworkError;
    use feagi_serialization::{FeagiByteContainer, SessionID};

    use super::EmbodimentTranslator;

    struct MockPublisher {
        state: FeagiEndpointState,
        published: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl FeagiServer for MockPublisher {
        fn poll(&mut self) -> &FeagiEndpointState {
            &self.state
        }

        fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
            Ok(())
        }

        fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
            Ok(())
        }

        fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
            self.state = FeagiEndpointState::Inactive;
            Ok(())
        }

        fn get_protocol(&self) -> TransportProtocolImplementation {
            TransportProtocolImplementation::Zmq
        }

        fn get_endpoint(&self) -> TransportProtocolEndpoint {
            panic!("endpoint not required for unit tests")
        }
    }

    impl FeagiServerPublisher for MockPublisher {
        fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
            self.published
                .lock()
                .expect("publisher mutex poisoned")
                .push(data.to_vec());
            Ok(())
        }

        fn as_boxed_publisher_properties(&self) -> Box<dyn FeagiServerPublisherProperties> {
            panic!("properties not required for unit tests")
        }
    }

    struct MockPuller {
        state: FeagiEndpointState,
        data: Vec<u8>,
    }

    impl FeagiServer for MockPuller {
        fn poll(&mut self) -> &FeagiEndpointState {
            &self.state
        }

        fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
            Ok(())
        }

        fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
            Ok(())
        }

        fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
            self.state = FeagiEndpointState::Inactive;
            Ok(())
        }

        fn get_protocol(&self) -> TransportProtocolImplementation {
            TransportProtocolImplementation::Zmq
        }

        fn get_endpoint(&self) -> TransportProtocolEndpoint {
            panic!("endpoint not required for unit tests")
        }
    }

    impl FeagiServerPuller for MockPuller {
        fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError> {
            Ok(self.data.as_slice())
        }
    }

    #[test]
    fn poll_sensor_server_returns_cached_data_when_endpoint_has_data() {
        let session_id = SessionID::new_random();
        let mut sensor_payload = FeagiByteContainer::new_empty();
        let _ = sensor_payload.set_session_id(session_id);
        let sensor_puller = Box::new(MockPuller {
            state: FeagiEndpointState::ActiveHasData,
            data: sensor_payload.get_byte_ref().to_vec(),
        });
        let motor_output = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let viz_output = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let motor_publisher = Box::new(MockPublisher {
            state: FeagiEndpointState::ActiveWaiting,
            published: Arc::clone(&motor_output),
        });
        let viz_publisher = Box::new(MockPublisher {
            state: FeagiEndpointState::ActiveWaiting,
            published: Arc::clone(&viz_output),
        });

        let mut translator =
            EmbodimentTranslator::new(session_id, motor_publisher, sensor_puller, viz_publisher);
        let result = translator
            .poll_sensor_server()
            .expect("polling sensor should not fail")
            .expect("sensor data should be present");

        assert_eq!(result.get_byte_ref(), sensor_payload.get_byte_ref());
        assert_eq!(result.get_session_id().ok(), Some(session_id));
    }

    #[test]
    fn send_motor_data_publishes_when_motor_endpoint_is_ready() {
        let session_id = SessionID::new_random();
        let motor_output = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let motor_publisher = Box::new(MockPublisher {
            state: FeagiEndpointState::ActiveWaiting,
            published: Arc::clone(&motor_output),
        });
        let viz_output = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let viz_publisher = Box::new(MockPublisher {
            state: FeagiEndpointState::ActiveWaiting,
            published: Arc::clone(&viz_output),
        });
        let sensor_puller = Box::new(MockPuller {
            state: FeagiEndpointState::ActiveWaiting,
            data: vec![],
        });
        let mut translator =
            EmbodimentTranslator::new(session_id, motor_publisher, sensor_puller, viz_publisher);

        let container = FeagiByteContainer::new_empty();
        translator
            .send_buffered_motor_data(&container)
            .expect("motor publish should succeed");

        let published = motor_output.lock().expect("publisher mutex poisoned");
        assert_eq!(published.len(), 1);
        assert_eq!(published[0], container.get_byte_ref().to_vec());
    }

    #[test]
    fn send_visualization_data_publishes_when_visualization_endpoint_is_ready() {
        let session_id = SessionID::new_random();
        let motor_output = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let motor_publisher = Box::new(MockPublisher {
            state: FeagiEndpointState::ActiveWaiting,
            published: Arc::clone(&motor_output),
        });
        let viz_output = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let viz_publisher = Box::new(MockPublisher {
            state: FeagiEndpointState::ActiveWaiting,
            published: Arc::clone(&viz_output),
        });
        let sensor_puller = Box::new(MockPuller {
            state: FeagiEndpointState::ActiveWaiting,
            data: vec![],
        });
        let mut translator =
            EmbodimentTranslator::new(session_id, motor_publisher, sensor_puller, viz_publisher);

        let container = FeagiByteContainer::new_empty();
        translator
            .send_visualization_data(&container)
            .expect("visualization publish should succeed");

        let published = viz_output.lock().expect("publisher mutex poisoned");
        assert_eq!(published.len(), 1);
        assert_eq!(published[0], container.get_byte_ref().to_vec());
    }
}