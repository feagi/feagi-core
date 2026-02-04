use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::FeagiAgentHandler;
use feagi_agent::registration::AgentCapabilities;
use feagi_config::FeagiConfig;
use feagi_io::core::protocol_implementations::ProtocolImplementation;
use feagi_serialization::SessionID;

fn build_test_config() -> FeagiConfig {
    let mut config = FeagiConfig::default();
    config.zmq.host = "192.0.2.10".to_string();
    config.ports.zmq_sensory_port = 4001;
    config.ports.zmq_motor_port = 4002;
    config.ports.zmq_visualization_port = 4003;
    config.websocket.host = "example.com".to_string();
    config.websocket.sensory_port = 9001;
    config.websocket.motor_port = 9002;
    config.websocket.visualization_port = 9003;
    config
}

#[test]
fn endpoints_are_built_from_config_for_zmq() {
    let config = build_test_config();
    let handler = FeagiAgentHandler::new_with_config(Box::new(DummyAuth {}), config);

    let sensory_endpoint = handler.build_capability_endpoint(
        &ProtocolImplementation::ZMQ,
        AgentCapabilities::SendSensorData,
    );
    let motor_endpoint = handler.build_capability_endpoint(
        &ProtocolImplementation::ZMQ,
        AgentCapabilities::ReceiveMotorData,
    );
    let viz_endpoint = handler.build_capability_endpoint(
        &ProtocolImplementation::ZMQ,
        AgentCapabilities::ReceiveNeuronVisualizations,
    );

    assert_eq!(sensory_endpoint, "tcp://192.0.2.10:4001");
    assert_eq!(motor_endpoint, "tcp://192.0.2.10:4002");
    assert_eq!(viz_endpoint, "tcp://192.0.2.10:4003");
}

#[test]
fn endpoints_are_built_from_config_for_websocket() {
    let config = build_test_config();
    let handler = FeagiAgentHandler::new_with_config(Box::new(DummyAuth {}), config);

    let sensory_endpoint = handler.build_capability_endpoint(
        &ProtocolImplementation::WebSocket,
        AgentCapabilities::SendSensorData,
    );
    let motor_endpoint = handler.build_capability_endpoint(
        &ProtocolImplementation::WebSocket,
        AgentCapabilities::ReceiveMotorData,
    );
    let viz_endpoint = handler.build_capability_endpoint(
        &ProtocolImplementation::WebSocket,
        AgentCapabilities::ReceiveNeuronVisualizations,
    );

    assert_eq!(sensory_endpoint, "ws://example.com:9001");
    assert_eq!(motor_endpoint, "ws://example.com:9002");
    assert_eq!(viz_endpoint, "ws://example.com:9003");
}

#[test]
fn unregistered_session_ids_are_rejected() {
    let config = build_test_config();
    let handler = FeagiAgentHandler::new_with_config(Box::new(DummyAuth {}), config);
    let session_id = SessionID::new_random();

    assert!(
        !handler.is_session_registered(&session_id),
        "Handler should reject unregistered session IDs"
    );
}
