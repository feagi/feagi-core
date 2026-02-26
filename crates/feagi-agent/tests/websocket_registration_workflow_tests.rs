#![cfg(all(
    feature = "agent-server",
    feature = "agent-client",
    feature = "agent-transport-websocket-std"
))]

use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use feagi_agent::clients::{AgentRegistrationStatus, CommandControlAgent};
use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken};
use feagi_io::protocol_implementations::websocket::websocket_std::{
    FeagiWebSocketClientRequesterProperties, FeagiWebSocketServerPublisherProperties,
    FeagiWebSocketServerPullerProperties, FeagiWebSocketServerRouterProperties,
};
use feagi_io::traits_and_enums::server::FeagiServerRouterProperties;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::traits_and_enums::shared::TransportProtocolEndpoint;
use feagi_io::AgentID;

fn reserve_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind ephemeral port");
    listener.local_addr().expect("failed to read local addr").port()
}

fn endpoint_as_string(endpoint: &TransportProtocolEndpoint) -> String {
    match endpoint {
        TransportProtocolEndpoint::WebSocket(url) => url.as_str().to_string(),
        TransportProtocolEndpoint::Zmq(url) => url.as_str().to_string(),
    }
}

#[test]
fn websocket_registration_workflow_succeeds_end_to_end() {
    let registration_port = reserve_free_port();
    let visualization_port = reserve_free_port();

    let registration_bind = format!("127.0.0.1:{registration_port}");
    let registration_remote = format!("ws://127.0.0.1:{registration_port}");
    let visualization_bind = format!("127.0.0.1:{visualization_port}");
    let visualization_remote = format!("ws://127.0.0.1:{visualization_port}");

    let mut handler = FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    );

    handler
        .add_and_start_command_control_server(Box::new(
            FeagiWebSocketServerRouterProperties::new_with_remote(
                &registration_bind,
                &registration_remote,
            )
            .expect("failed to create websocket router properties"),
        ))
        .expect("failed to start websocket command/control router");

    handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&visualization_bind, &visualization_remote)
            .expect("failed to create websocket visualization publisher properties"),
    ));

    let registration_remote_for_client = registration_remote.clone();
    let (client_result_tx, client_result_rx) = mpsc::channel::<Result<(AgentID, String), String>>();
    let client_thread = thread::spawn(move || {
        let result: Result<(AgentID, String), String> = (|| {
            let requester_properties = Box::new(
                FeagiWebSocketClientRequesterProperties::new(&registration_remote_for_client)
                    .map_err(|e| format!("failed to create websocket requester properties: {e}"))?,
            );
            let mut client = CommandControlAgent::new(requester_properties);

            client
                .request_connect()
                .map_err(|e| format!("client failed to connect: {e}"))?;
            client
                .request_registration(
                    AgentDescriptor::new("neuraville", "ws-registration-test", 1)
                        .map_err(|e| format!("invalid descriptor: {e}"))?,
                    AuthToken::new([0u8; 32]),
                    vec![AgentCapabilities::ReceiveNeuronVisualizations],
                )
                .map_err(|e| format!("client failed to send registration request: {e}"))?;

            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                client
                    .poll_for_messages()
                    .map_err(|e| format!("client poll_for_messages failed: {e}"))?;

                if let AgentRegistrationStatus::Registered(agent_id, endpoints) =
                    client.registration_status()
                {
                    let viz_endpoint = endpoint_as_string(
                        endpoints
                            .get(&AgentCapabilities::ReceiveNeuronVisualizations)
                            .ok_or_else(|| {
                                "missing visualization endpoint in registration response"
                                    .to_string()
                            })?,
                    );
                    return Ok((*agent_id, viz_endpoint));
                }

                if Instant::now() >= deadline {
                    return Err(format!(
                        "timed out waiting for websocket registration response; status={:?}",
                        client.registration_status()
                    ));
                }

                thread::sleep(Duration::from_millis(2));
            }
        })();
        let _ = client_result_tx.send(result);
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        handler
            .poll_command_and_control()
            .expect("server poll_command_and_control failed");

        if let Ok(client_result) = client_result_rx.try_recv() {
            let (agent_id, returned_viz_endpoint) =
                client_result.expect("client registration flow failed");

            assert_eq!(
                returned_viz_endpoint,
                visualization_remote,
                "server should return websocket visualization endpoint"
            );

            let registered_agents = handler.get_all_registered_agents();
            assert!(
                registered_agents.contains_key(&agent_id),
                "server should track the newly registered websocket session"
            );
            let (descriptor, capabilities) = registered_agents
                .get(&agent_id)
                .expect("registered agent should exist");
            assert_eq!(descriptor.agent_name(), "ws-registration-test");
            assert!(
                capabilities.contains(&AgentCapabilities::ReceiveNeuronVisualizations),
                "server should persist requested visualization capability"
            );

            // Keep compile-time use of AgentID explicit to ensure workflow tracks concrete sessions.
            let _session_copy: AgentID = agent_id;
            client_thread.join().expect("client thread panicked");
            break;
        }

        if Instant::now() >= deadline {
            let _ = client_thread.join();
            panic!(
                "timed out waiting for websocket registration response while server was polling"
            );
        }

        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn websocket_registration_times_out_when_server_does_not_respond() {
    let registration_port = reserve_free_port();
    let registration_bind = format!("127.0.0.1:{registration_port}");
    let registration_remote = format!("ws://127.0.0.1:{registration_port}");

    let router_props =
        FeagiWebSocketServerRouterProperties::new_with_remote(&registration_bind, &registration_remote)
            .expect("failed to create websocket router properties");
    let mut router = router_props.as_boxed_server_router();
    router
        .request_start()
        .expect("failed to start websocket router");

    let registration_remote_for_client = registration_remote.clone();
    let (client_result_tx, client_result_rx) = mpsc::channel::<Result<(), String>>();
    let client_thread = thread::spawn(move || {
        let result: Result<(), String> = (|| {
            let requester_properties = Box::new(
                FeagiWebSocketClientRequesterProperties::new(&registration_remote_for_client)
                    .map_err(|e| format!("failed to create websocket requester properties: {e}"))?,
            );
            let mut client = CommandControlAgent::new(requester_properties);
            client
                .request_connect()
                .map_err(|e| format!("client failed to connect: {e}"))?;
            client
                .request_registration(
                    AgentDescriptor::new("neuraville", "ws-no-reply-test", 1)
                        .map_err(|e| format!("invalid descriptor: {e}"))?,
                    AuthToken::new([0u8; 32]),
                    vec![AgentCapabilities::ReceiveNeuronVisualizations],
                )
                .map_err(|e| format!("client failed to send registration request: {e}"))?;

            let deadline = Instant::now() + Duration::from_millis(1200);
            loop {
                client
                    .poll_for_messages()
                    .map_err(|e| format!("client poll_for_messages failed: {e}"))?;

                if matches!(client.registration_status(), AgentRegistrationStatus::Registered(_, _)) {
                    return Err("client unexpectedly registered without server response".to_string());
                }
                if Instant::now() >= deadline {
                    // Expected: no registration response received within deadline.
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(2));
            }
        })();
        let _ = client_result_tx.send(result);
    });

    let mut saw_request = false;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match router.poll().clone() {
            FeagiEndpointState::ActiveHasData => {
                // Consume the registration request but intentionally do not publish a response.
                let (_session_id, _payload) = router
                    .consume_retrieved_request()
                    .expect("failed to consume websocket request");
                saw_request = true;
            }
            FeagiEndpointState::Errored(err) => {
                panic!("websocket router errored while running timeout scenario: {err}");
            }
            _ => {}
        }

        if let Ok(client_result) = client_result_rx.try_recv() {
            client_result.expect("client timeout expectation failed");
            assert!(
                saw_request,
                "server should receive the registration request in timeout scenario"
            );
            client_thread.join().expect("client thread panicked");
            break;
        }

        if Instant::now() >= deadline {
            let _ = client_thread.join();
            panic!("timed out waiting for websocket timeout scenario to complete");
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn websocket_registration_returns_all_requested_capability_endpoints() {
    let registration_port = reserve_free_port();
    let sensory_port = reserve_free_port();
    let motor_port = reserve_free_port();
    let visualization_port = reserve_free_port();

    let registration_bind = format!("127.0.0.1:{registration_port}");
    let registration_remote = format!("ws://127.0.0.1:{registration_port}");
    let sensory_bind = format!("127.0.0.1:{sensory_port}");
    let sensory_remote = format!("ws://127.0.0.1:{sensory_port}");
    let motor_bind = format!("127.0.0.1:{motor_port}");
    let motor_remote = format!("ws://127.0.0.1:{motor_port}");
    let visualization_bind = format!("127.0.0.1:{visualization_port}");
    let visualization_remote = format!("ws://127.0.0.1:{visualization_port}");

    let mut handler = FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    );
    handler
        .add_and_start_command_control_server(Box::new(
            FeagiWebSocketServerRouterProperties::new_with_remote(
                &registration_bind,
                &registration_remote,
            )
            .expect("failed to create websocket router properties"),
        ))
        .expect("failed to start websocket command/control router");
    handler.add_puller_server(Box::new(
        FeagiWebSocketServerPullerProperties::new_with_remote(&sensory_bind, &sensory_remote)
            .expect("failed to create websocket sensory puller properties"),
    ));
    // Order matters: visualization endpoint resolver uses last matching publisher.
    handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&motor_bind, &motor_remote)
            .expect("failed to create websocket motor publisher properties"),
    ));
    handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&visualization_bind, &visualization_remote)
            .expect("failed to create websocket visualization publisher properties"),
    ));

    let registration_remote_for_client = registration_remote.clone();
    let (client_result_tx, client_result_rx) =
        mpsc::channel::<Result<(AgentID, String, String, String), String>>();
    let client_thread = thread::spawn(move || {
        let result: Result<(AgentID, String, String, String), String> = (|| {
            let requester_properties = Box::new(
                FeagiWebSocketClientRequesterProperties::new(&registration_remote_for_client)
                    .map_err(|e| format!("failed to create websocket requester properties: {e}"))?,
            );
            let mut client = CommandControlAgent::new(requester_properties);
            client
                .request_connect()
                .map_err(|e| format!("client failed to connect: {e}"))?;
            client
                .request_registration(
                    AgentDescriptor::new("neuraville", "ws-registration-all-caps-test", 1)
                        .map_err(|e| format!("invalid descriptor: {e}"))?,
                    AuthToken::new([0u8; 32]),
                    vec![
                        AgentCapabilities::SendSensorData,
                        AgentCapabilities::ReceiveMotorData,
                        AgentCapabilities::ReceiveNeuronVisualizations,
                    ],
                )
                .map_err(|e| format!("client failed to send registration request: {e}"))?;

            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                client
                    .poll_for_messages()
                    .map_err(|e| format!("client poll_for_messages failed: {e}"))?;
                if let AgentRegistrationStatus::Registered(agent_id, endpoints) =
                    client.registration_status()
                {
                    let sensory = endpoint_as_string(
                        endpoints
                            .get(&AgentCapabilities::SendSensorData)
                            .ok_or_else(|| "missing sensory endpoint".to_string())?,
                    );
                    let motor = endpoint_as_string(
                        endpoints
                            .get(&AgentCapabilities::ReceiveMotorData)
                            .ok_or_else(|| "missing motor endpoint".to_string())?,
                    );
                    let viz = endpoint_as_string(
                        endpoints
                            .get(&AgentCapabilities::ReceiveNeuronVisualizations)
                            .ok_or_else(|| "missing visualization endpoint".to_string())?,
                    );
                    return Ok((*agent_id, sensory, motor, viz));
                }
                if Instant::now() >= deadline {
                    return Err("timed out waiting for all-capabilities registration".to_string());
                }
                thread::sleep(Duration::from_millis(2));
            }
        })();
        let _ = client_result_tx.send(result);
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        handler
            .poll_command_and_control()
            .expect("server poll_command_and_control failed");
        if let Ok(client_result) = client_result_rx.try_recv() {
            let (agent_id, sensory_endpoint, motor_endpoint, viz_endpoint) =
                client_result.expect("client all-capabilities registration failed");
            assert_eq!(sensory_endpoint, sensory_remote, "sensory endpoint mismatch");
            assert_eq!(motor_endpoint, motor_remote, "motor endpoint mismatch");
            assert_eq!(
                viz_endpoint, visualization_remote,
                "visualization endpoint mismatch"
            );

            let registered_agents = handler.get_all_registered_agents();
            let (_descriptor, capabilities) = registered_agents
                .get(&agent_id)
                .expect("registered agent should exist");
            assert!(capabilities.contains(&AgentCapabilities::SendSensorData));
            assert!(capabilities.contains(&AgentCapabilities::ReceiveMotorData));
            assert!(capabilities.contains(&AgentCapabilities::ReceiveNeuronVisualizations));
            client_thread.join().expect("client thread panicked");
            break;
        }
        if Instant::now() >= deadline {
            let _ = client_thread.join();
            panic!("timed out waiting for all-capabilities registration scenario");
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn websocket_deregistration_removes_registered_agent_from_server() {
    let registration_port = reserve_free_port();
    let visualization_port = reserve_free_port();

    let registration_bind = format!("127.0.0.1:{registration_port}");
    let registration_remote = format!("ws://127.0.0.1:{registration_port}");
    let visualization_bind = format!("127.0.0.1:{visualization_port}");
    let visualization_remote = format!("ws://127.0.0.1:{visualization_port}");

    let mut handler = FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    );
    handler
        .add_and_start_command_control_server(Box::new(
            FeagiWebSocketServerRouterProperties::new_with_remote(
                &registration_bind,
                &registration_remote,
            )
            .expect("failed to create websocket router properties"),
        ))
        .expect("failed to start websocket command/control router");
    handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&visualization_bind, &visualization_remote)
            .expect("failed to create websocket visualization publisher properties"),
    ));

    let registration_remote_for_client = registration_remote.clone();
    let (client_result_tx, client_result_rx) = mpsc::channel::<Result<AgentID, String>>();
    let client_thread = thread::spawn(move || {
        let result: Result<AgentID, String> = (|| {
            let requester_properties = Box::new(
                FeagiWebSocketClientRequesterProperties::new(&registration_remote_for_client)
                    .map_err(|e| format!("failed to create websocket requester properties: {e}"))?,
            );
            let mut client = CommandControlAgent::new(requester_properties);
            client
                .request_connect()
                .map_err(|e| format!("client failed to connect: {e}"))?;
            client
                .request_registration(
                    AgentDescriptor::new("neuraville", "ws-deregistration-test", 1)
                        .map_err(|e| format!("invalid descriptor: {e}"))?,
                    AuthToken::new([0u8; 32]),
                    vec![AgentCapabilities::ReceiveNeuronVisualizations],
                )
                .map_err(|e| format!("client failed to send registration request: {e}"))?;

            let reg_deadline = Instant::now() + Duration::from_secs(5);
            let registered_agent: AgentID = loop {
                client
                    .poll_for_messages()
                    .map_err(|e| format!("client poll_for_messages failed: {e}"))?;
                if let AgentRegistrationStatus::Registered(agent_id, _) = client.registration_status() {
                    break *agent_id;
                }
                if Instant::now() >= reg_deadline {
                    return Err("timed out waiting for registration before deregistration".to_string());
                }
                thread::sleep(Duration::from_millis(2));
            };

            client
                .request_deregistration(Some("test cleanup".to_string()))
                .map_err(|e| format!("client failed to send deregistration request: {e}"))?;

            let dereg_deadline = Instant::now() + Duration::from_secs(5);
            loop {
                client
                    .poll_for_messages()
                    .map_err(|e| format!("client poll_for_messages failed during dereg: {e}"))?;
                if matches!(client.registration_status(), AgentRegistrationStatus::NotRegistered) {
                    return Ok(registered_agent);
                }
                if Instant::now() >= dereg_deadline {
                    return Err("timed out waiting for deregistration acknowledgement".to_string());
                }
                thread::sleep(Duration::from_millis(2));
            }
        })();
        let _ = client_result_tx.send(result);
    });

    let deadline = Instant::now() + Duration::from_secs(6);
    let agent_id: AgentID = loop {
        handler
            .poll_command_and_control()
            .expect("server poll_command_and_control failed");

        if let Ok(client_result) = client_result_rx.try_recv() {
            break client_result.expect("client deregistration flow failed");
        }
        if Instant::now() >= deadline {
            let _ = client_thread.join();
            panic!("timed out waiting for client deregistration scenario");
        }
        thread::sleep(Duration::from_millis(2));
    };
    let server_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        handler
            .poll_command_and_control()
            .expect("server poll_command_and_control failed");
        if !handler.get_all_registered_agents().contains_key(&agent_id) {
            client_thread.join().expect("client thread panicked");
            break;
        }
        if Instant::now() >= server_deadline {
            let _ = client_thread.join();
            panic!("server still retains agent after successful deregistration");
        }
        thread::sleep(Duration::from_millis(2));
    }
}
