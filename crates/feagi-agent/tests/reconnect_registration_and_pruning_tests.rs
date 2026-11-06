//! Reconnect registration ordering and stale-agent pruning over a live in-process server.
//!
//! These tests drive a real [`FeagiAgentHandler`] (ZMQ transports on loopback, the transport the
//! Python and Rust SDKs use) with real [`CommandControlAgent`] clients, stepped from one thread so
//! ordering is deterministic. ZMQ is used because its client queues a request and drops what is
//! still queued when the socket is disconnected (linger 0), which is the behaviour the rebuild
//! ordering must survive; the WebSocket client sends synchronously and cannot show it.
//!
//! Registration: the server rejects a registration for a descriptor that already has a session
//! with `AlreadyRegistered` while that session's last command/control message is younger than
//! twice `stale_check_interval`. A client that rebuilds its session on a new socket must therefore
//! deregister the old session and wait for the acknowledgement before it registers again. If the
//! link is dead and no acknowledgement ever arrives, the new registration replaces the old session
//! once the guard window has passed.
//!
//! Pruning: the stale pruner counts only what the client sends to FEAGI. Data FEAGI pushes to the
//! client must not keep a dead client registered, and a client that streams sensory data must stay
//! registered even if it rarely sends a heartbeat.

#![cfg(all(
    feature = "agent-server",
    feature = "agent-client",
    feature = "agent-transport-zmq"
))]

use std::net::TcpListener;
use std::thread::sleep;
use std::time::{Duration, Instant};

use feagi_agent::clients::{AgentRegistrationStatus, CommandControlAgent};
use feagi_agent::command_and_control::FeagiMessage;
use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use feagi_io::protocol_implementations::zmq::{
    FeagiZmqClientPusherProperties, FeagiZmqClientRequesterProperties,
    FeagiZmqServerPublisherProperties, FeagiZmqServerPullerProperties,
    FeagiZmqServerRouterProperties,
};
use feagi_io::traits_and_enums::client::FeagiClientPusherProperties;
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;
use feagi_structures::FeagiJSON;

/// Step interval for the single-threaded drivers. Not a tuning knob: it only bounds busy-looping.
const STEP: Duration = Duration::from_millis(2);
/// Upper bound for any single wait in these tests.
const WAIT_LIMIT: Duration = Duration::from_secs(5);

/// Server-side timing used by the registration tests. The duplicate-registration guard window is
/// twice the stale scan interval, so every wait derived from it comes from this one value.
const REGISTRATION_STALE_CHECK_INTERVAL: Duration = Duration::from_millis(500);

fn guard_window() -> Duration {
    REGISTRATION_STALE_CHECK_INTERVAL * 2
}

fn reserve_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind ephemeral port")
        .local_addr()
        .expect("failed to read local addr")
        .port()
}

fn endpoint_url(endpoint: &TransportProtocolEndpoint) -> String {
    match endpoint {
        TransportProtocolEndpoint::WebSocket(url) => url.as_str().to_string(),
        TransportProtocolEndpoint::Zmq(url) => url.as_str().to_string(),
    }
}

/// An in-process FEAGI server: one command/control router plus optional data endpoints.
struct TestServer {
    handler: FeagiAgentHandler,
    registration_remote: String,
}

impl TestServer {
    fn new(config: AgentLivenessConfig, with_sensory: bool, with_visualization: bool) -> Self {
        let registration_port = reserve_free_port();
        let registration_remote = format!("tcp://127.0.0.1:{registration_port}");

        let mut handler =
            FeagiAgentHandler::new_with_liveness_config(Box::new(DummyAuth {}), config);
        handler
            .add_and_start_command_control_server(Box::new(
                FeagiZmqServerRouterProperties::new(&registration_remote, &registration_remote)
                    .expect("router properties"),
            ))
            .expect("start command/control router");

        if with_sensory {
            let port = reserve_free_port();
            handler.add_puller_server(Box::new(
                FeagiZmqServerPullerProperties::new(
                    &format!("tcp://127.0.0.1:{port}"),
                    &format!("tcp://127.0.0.1:{port}"),
                )
                .expect("puller properties"),
            ));
        }
        if with_visualization {
            let port = reserve_free_port();
            handler.add_publisher_server(Box::new(
                FeagiZmqServerPublisherProperties::new(
                    &format!("tcp://127.0.0.1:{port}"),
                    &format!("tcp://127.0.0.1:{port}"),
                )
                .expect("publisher properties"),
            ));
        }

        Self {
            handler,
            registration_remote,
        }
    }

    fn new_client(&self) -> CommandControlAgent {
        CommandControlAgent::new(Box::new(
            FeagiZmqClientRequesterProperties::new(&self.registration_remote)
                .expect("requester properties"),
        ))
    }

    /// Run one server polling pass.
    fn step(&mut self) {
        self.handler
            .poll_command_and_control()
            .expect("server poll_command_and_control failed");
    }

    /// Step the server for `duration` without any client activity.
    fn run_for(&mut self, duration: Duration) {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            self.step();
            sleep(STEP);
        }
    }

    /// Connect `client` and register it, stepping the server until the response arrives.
    ///
    /// Returns the client-side error (for example `Client already registered!`) when the server
    /// rejects the registration.
    fn register(
        &mut self,
        client: &mut CommandControlAgent,
        descriptor: &AgentDescriptor,
        capabilities: Vec<AgentCapabilities>,
    ) -> Result<(AgentID, Vec<(AgentCapabilities, TransportProtocolEndpoint)>), FeagiAgentError>
    {
        client.request_connect()?;
        client.request_registration(descriptor.clone(), AuthToken::new([0u8; 32]), capabilities)?;
        let deadline = Instant::now() + WAIT_LIMIT;
        loop {
            self.step();
            client.poll_for_messages()?;
            if let AgentRegistrationStatus::Registered(agent_id, endpoints) =
                client.registration_status()
            {
                let endpoints = endpoints
                    .iter()
                    .map(|(capability, endpoint)| (*capability, endpoint.clone()))
                    .collect();
                return Ok((*agent_id, endpoints));
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for a registration response"
            );
            sleep(STEP);
        }
    }

    /// Send one heartbeat and step until the server's reply reaches the client, so the server
    /// has certainly recorded the command/control activity.
    fn heartbeat(&mut self, client: &mut CommandControlAgent) {
        client.send_heartbeat().expect("send heartbeat");
        let deadline = Instant::now() + WAIT_LIMIT;
        loop {
            self.step();
            let (_, message) = client.poll_for_messages().expect("client poll");
            if matches!(message, Some(FeagiMessage::HeartBeat)) {
                return;
            }
            assert!(Instant::now() < deadline, "timed out waiting for heartbeat");
            sleep(STEP);
        }
    }

    /// Request deregistration and step until the server acknowledges it.
    fn deregister_and_wait_for_ack(&mut self, client: &mut CommandControlAgent) {
        client
            .request_deregistration(Some("rebuild".to_string()))
            .expect("request deregistration");
        let deadline = Instant::now() + WAIT_LIMIT;
        loop {
            self.step();
            client.poll_for_messages().expect("client poll");
            if matches!(
                client.registration_status(),
                AgentRegistrationStatus::NotRegistered
            ) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for the deregistration acknowledgement"
            );
            sleep(STEP);
        }
    }

    fn registered_ids(&self) -> Vec<AgentID> {
        self.handler
            .get_all_registered_agents()
            .keys()
            .copied()
            .collect()
    }
}

fn descriptor(agent_name: &str) -> AgentDescriptor {
    AgentDescriptor::new("neuraville", agent_name, 1).expect("descriptor inputs must be valid")
}

fn registration_config() -> AgentLivenessConfig {
    AgentLivenessConfig {
        heartbeat_timeout: Duration::from_secs(60),
        stale_check_interval: REGISTRATION_STALE_CHECK_INTERVAL,
    }
}

const VIZ: AgentCapabilities = AgentCapabilities::ReceiveNeuronVisualizations;

//region Registration ordering on reconnect

#[test]
fn registration_handled_before_the_old_deregistration_is_rejected_as_already_registered() {
    let mut server = TestServer::new(registration_config(), false, true);
    let agent = descriptor("rebuild-order-old");

    let mut old_client = server.new_client();
    server
        .register(&mut old_client, &agent, vec![VIZ])
        .expect("first registration");
    // A heartbeat is in flight right before the rebuild, so the old session is fresh.
    server.heartbeat(&mut old_client);

    // The previous rebuild queued the deregistration, did not wait for its acknowledgement and
    // registered again on a fresh socket. The two sockets have no ordering at the server, so
    // the new registration can be handled while the old session is still registered and fresh.
    // That interleaving is modelled deterministically here: the new registration is handled
    // first, and the old session's deregistration only follows afterwards.
    let mut new_client = server.new_client();
    let outcome = server.register(&mut new_client, &agent, vec![VIZ]);

    let error = outcome.expect_err("a registration inside the guard window must be rejected");
    assert!(
        error.to_string().contains("already registered"),
        "expected AlreadyRegistered, got: {error}"
    );
    assert_eq!(
        server.registered_ids().len(),
        1,
        "the rejected registration must not create a second session"
    );
}

#[test]
fn new_rebuild_ordering_registers_on_the_first_try() {
    let mut server = TestServer::new(registration_config(), false, true);
    let agent = descriptor("rebuild-order-new");

    let mut old_client = server.new_client();
    let (old_id, _) = server
        .register(&mut old_client, &agent, vec![VIZ])
        .expect("first registration");
    // The same in-flight heartbeat as in the old-ordering test.
    server.heartbeat(&mut old_client);

    server.deregister_and_wait_for_ack(&mut old_client);
    old_client
        .request_disconnect()
        .expect("disconnect old socket");
    drop(old_client);

    let mut new_client = server.new_client();
    let (new_id, _) = server
        .register(&mut new_client, &agent, vec![VIZ])
        .expect("re-registration must succeed on the first try");

    assert_ne!(old_id, new_id);
    assert_eq!(server.registered_ids(), vec![new_id]);
}

#[test]
fn dead_link_registration_replaces_the_old_session_after_the_ack_budget() {
    let mut server = TestServer::new(registration_config(), false, true);
    let agent = descriptor("rebuild-dead-link");

    let mut old_client = server.new_client();
    let (old_id, _) = server
        .register(&mut old_client, &agent, vec![VIZ])
        .expect("first registration");
    server.heartbeat(&mut old_client);

    // Dead link: the deregistration never reaches the server, so no acknowledgement arrives.
    // The client waits out its ack budget (the server keeps serving), then proceeds.
    let ack_budget = guard_window() + Duration::from_millis(200);
    server.run_for(ack_budget);
    assert!(
        matches!(
            old_client.registration_status(),
            AgentRegistrationStatus::Registered(_, _)
        ),
        "no acknowledgement can arrive on a dead link"
    );
    drop(old_client);

    let mut new_client = server.new_client();
    let (new_id, _) = server
        .register(&mut new_client, &agent, vec![VIZ])
        .expect("registration after the ack budget must replace the old session");

    assert_ne!(old_id, new_id);
    assert_eq!(
        server.registered_ids(),
        vec![new_id],
        "the old session must be replaced, not kept alongside"
    );
}

//endregion

//region Stale pruning counts only inbound traffic

const PRUNE_HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(400);

fn pruning_config() -> AgentLivenessConfig {
    AgentLivenessConfig {
        heartbeat_timeout: PRUNE_HEARTBEAT_TIMEOUT,
        stale_check_interval: Duration::from_millis(20),
    }
}

#[test]
fn pruner_drops_a_client_that_only_receives_pushes() {
    let mut server = TestServer::new(pruning_config(), false, true);
    let mut client = server.new_client();
    let (agent_id, _) = server
        .register(&mut client, &descriptor("push-only"), vec![VIZ])
        .expect("registration");

    // The client now goes quiet: it is never polled and never sends. FEAGI keeps pushing to it.
    let payload = FeagiByteContainer::new_empty();
    let mut pushes_sent = 0usize;
    let deadline = Instant::now() + PRUNE_HEARTBEAT_TIMEOUT * 4;
    while Instant::now() < deadline && server.registered_ids().contains(&agent_id) {
        server.step();
        server
            .handler
            .poll_agent_visualizers()
            .expect("poll visualizers");
        if server
            .handler
            .send_visualization_data_to_agent(agent_id, &payload)
            .is_ok()
        {
            pushes_sent += 1;
        }
        sleep(STEP);
    }

    assert!(
        pushes_sent > 0,
        "the test must actually push data to the client"
    );
    assert!(
        !server.registered_ids().contains(&agent_id),
        "a client that sends nothing must be pruned although FEAGI keeps pushing to it \
         ({pushes_sent} pushes sent)"
    );
}

#[test]
fn pruner_keeps_a_client_that_streams_sensory_data_without_heartbeats() {
    let mut server = TestServer::new(pruning_config(), true, false);
    let mut client = server.new_client();
    let (agent_id, endpoints) = server
        .register(
            &mut client,
            &descriptor("sensory-streamer"),
            vec![AgentCapabilities::SendSensorData],
        )
        .expect("registration");
    let sensory_endpoint = endpoints
        .iter()
        .find(|(capability, _)| *capability == AgentCapabilities::SendSensorData)
        .map(|(_, endpoint)| endpoint_url(endpoint))
        .expect("sensory endpoint in the registration response");

    let mut pusher = FeagiZmqClientPusherProperties::new(&sensory_endpoint)
        .expect("pusher properties")
        .as_boxed_client_pusher();
    pusher.request_connect().expect("sensory connect");

    // Stream sensory frames for three heartbeat timeouts. The client sends no heartbeat at all
    // after registering, so only the sensory traffic can keep it registered.
    // The server drops empty containers (no structures) before they count as sensory data, so
    // the frame carries one (empty) JSON structure.
    let mut frame = FeagiByteContainer::new_empty();
    frame
        .overwrite_byte_data_with_single_struct_data(&FeagiJSON::new_empty(), 0)
        .expect("build sensory frame");
    let mut frames_received = 0usize;
    let end = Instant::now() + PRUNE_HEARTBEAT_TIMEOUT * 3;
    while Instant::now() < end {
        server.step();
        if matches!(pusher.poll(), FeagiEndpointState::ActiveWaiting) {
            pusher
                .publish_data(frame.get_byte_ref())
                .expect("publish sensory frame");
        }
        if server
            .handler
            .poll_agent_sensors()
            .expect("poll sensors")
            .is_some()
        {
            frames_received += 1;
        }
        assert!(
            server.registered_ids().contains(&agent_id),
            "a client streaming sensory data must not be pruned (frames so far: {frames_received})"
        );
        sleep(STEP);
    }
    assert!(
        frames_received > 0,
        "the test must actually deliver sensory frames to the server"
    );

    // Once the stream stops the same client goes stale: the pruner is still active.
    let deadline = Instant::now() + PRUNE_HEARTBEAT_TIMEOUT * 4;
    while Instant::now() < deadline && server.registered_ids().contains(&agent_id) {
        server.step();
        sleep(STEP);
    }
    assert!(
        !server.registered_ids().contains(&agent_id),
        "a silent client must be pruned once its sensory stream stops"
    );
}

//endregion
