//! Automatic recovery of `TokioEmbodimentAgent` against a live in-process server.
//!
//! The server runs on its own thread (ZMQ on loopback) because a re-registration blocks the
//! calling thread while it waits for FEAGI. A forced deregistration is what a genome load does:
//! FEAGI drops every session without telling the agents, and answers the next heartbeat with
//! `NotRegistered`. Supervisor time is mocked (`supervised_tick_at`), so the 30 s slow retry is
//! checked without waiting 30 s.

#![cfg(all(
    feature = "agent-server",
    feature = "agent-client",
    feature = "agent-client-asynchelper-tokio",
    feature = "agent-transport-zmq"
))]

use std::net::TcpListener;
use std::sync::mpsc::{channel, Sender};
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, Instant};

use feagi_agent::clients::async_helpers::tokio_generic_implementations::{
    TokioDriverConfig, TokioEmbodimentAgent,
};
use feagi_agent::clients::recovery::{
    ReconnectPolicyConfig, SessionRecoveryState, SessionSupervisor, SupervisorConfig,
};
use feagi_agent::clients::SessionTimingConfig;
use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use feagi_io::protocol_implementations::zmq::{
    FeagiZmqClientRequesterProperties, FeagiZmqServerPublisherProperties,
    FeagiZmqServerPullerProperties, FeagiZmqServerRouterProperties,
};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};

const STEP: Duration = Duration::from_millis(2);
const WAIT_LIMIT: Duration = Duration::from_secs(5);
/// Real-time bound of one re-registration attempt in these tests.
const REBUILD_TIMEOUT_MS: u64 = 200;

fn reserve_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

#[derive(Clone)]
struct Ports {
    command: u16,
    sensory: u16,
    motor: u16,
}

impl Ports {
    fn new() -> Self {
        Self {
            command: reserve_free_port(),
            sensory: reserve_free_port(),
            motor: reserve_free_port(),
        }
    }
    fn command_url(&self) -> String {
        format!("tcp://127.0.0.1:{}", self.command)
    }
}

type ServerJob = Box<dyn FnOnce(&mut FeagiAgentHandler) + Send>;

/// A FEAGI-side handler on its own thread, polled continuously. `stop` drops it (server gone).
struct ServerThread {
    jobs: Sender<ServerJob>,
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl ServerThread {
    fn start(ports: &Ports) -> Self {
        let ports = ports.clone();
        let (jobs, job_rx) = channel::<ServerJob>();
        let (stop, stop_rx) = channel::<()>();
        let thread = std::thread::spawn(move || {
            let mut handler = FeagiAgentHandler::new_with_liveness_config(
                Box::new(DummyAuth {}),
                AgentLivenessConfig {
                    heartbeat_timeout: Duration::from_secs(60),
                    stale_check_interval: Duration::from_millis(100),
                },
            );
            let cmd = ports.command_url();
            handler
                .add_and_start_command_control_server(Box::new(
                    FeagiZmqServerRouterProperties::new(&cmd, &cmd).expect("router"),
                ))
                .expect("start router");
            let sensory = format!("tcp://127.0.0.1:{}", ports.sensory);
            handler.add_puller_server(Box::new(
                FeagiZmqServerPullerProperties::new(&sensory, &sensory).expect("puller"),
            ));
            let motor = format!("tcp://127.0.0.1:{}", ports.motor);
            handler.add_publisher_server(Box::new(
                FeagiZmqServerPublisherProperties::new(&motor, &motor).expect("publisher"),
            ));
            loop {
                if stop_rx.try_recv().is_ok() {
                    return;
                }
                while let Ok(job) = job_rx.try_recv() {
                    job(&mut handler);
                }
                handler.poll_command_and_control().expect("server poll");
                let _ = handler.poll_agent_sensors();
                let _ = handler.poll_agent_motors();
                sleep(STEP);
            }
        });
        Self {
            jobs,
            stop,
            thread: Some(thread),
        }
    }

    /// Run `f` on the server thread and return its result.
    fn ask<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut FeagiAgentHandler) -> T + Send + 'static,
    ) -> T {
        let (tx, rx) = channel();
        self.jobs
            .send(Box::new(move |h| {
                let _ = tx.send(f(h));
            }))
            .expect("server thread alive");
        rx.recv_timeout(WAIT_LIMIT).expect("server answered")
    }

    fn registered_count(&self) -> usize {
        self.ask(|h| h.get_all_registered_agents().len())
    }

    fn stop(&mut self) {
        let _ = self.stop.send(());
        if let Some(t) = self.thread.take() {
            t.join().expect("server thread");
        }
    }
}

impl Drop for ServerThread {
    fn drop(&mut self) {
        self.stop();
    }
}

fn descriptor() -> AgentDescriptor {
    AgentDescriptor::new("neuraville", "recovery-test", 1).expect("descriptor")
}

fn register_infrared(agent: &TokioEmbodimentAgent, unit: u16, channels: u32) {
    agent
        .get_embodiment()
        .get_sensor_cache()
        .infrared_register(
            CorticalUnitIndex::from(unit),
            CorticalChannelCount::new(channels).unwrap(),
            FrameChangeHandling::Absolute,
            NeuronDepth::new(10).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .expect("register infrared");
}

/// Connect an agent with one registered sensor, as an app would.
fn connected_agent(ports: &Ports) -> TokioEmbodimentAgent {
    let mut agent = TokioEmbodimentAgent::new_unconnected(
        Box::new(FeagiZmqClientRequesterProperties::new(&ports.command_url()).expect("requester")),
        descriptor(),
        AuthToken::new([0u8; 32]),
        vec![
            AgentCapabilities::SendSensorData,
            AgentCapabilities::ReceiveMotorData,
        ],
        TokioDriverConfig {
            poll_interval: STEP,
            timing: SessionTimingConfig {
                heartbeat_interval_ms: 20,
                registration_deadline_ms: Some(REBUILD_TIMEOUT_MS),
            },
            sensory_rate_negotiation: None,
        },
    );
    register_infrared(&agent, 0, 1);
    agent
        .connect_and_register_spin()
        .expect("first registration");
    agent
}

fn supervisor() -> SessionSupervisor {
    SessionSupervisor::new(SupervisorConfig::new(
        ReconnectPolicyConfig {
            cooldown_ms: 1_000,
            max_consecutive_failures: 3,
            trigger_on_session_changed: false,
            trigger_on_genome_changed: false,
            trigger_on_genome_load_completed: false,
            trigger_on_back_online: false,
            trigger_on_brain_ready: false,
            trigger_on_transport_send_failed: true,
        },
        REBUILD_TIMEOUT_MS,
    ))
}

/// What a genome load does to the agent: FEAGI drops the session; the next heartbeat is
/// answered `NotRegistered`. Ticks the agent until it reports the typed error.
fn end_session_like_a_genome_load(server: &ServerThread, agent: &mut TokioEmbodimentAgent) {
    let dropped = server.ask(|h| h.force_deregister_all_agents("forced by genome transition"));
    assert_eq!(dropped.len(), 1);
    let deadline = Instant::now() + WAIT_LIMIT;
    loop {
        match agent.tick() {
            Err(FeagiAgentError::NotRegistered(_)) => return,
            Err(other) => panic!("expected the typed NotRegistered error, got {other:?}"),
            Ok(()) => {}
        }
        assert!(
            Instant::now() < deadline,
            "agent never learned its session ended"
        );
        sleep(STEP);
    }
}

#[test]
fn forced_deregistration_is_recovered_with_the_same_device_registrations() {
    let ports = Ports::new();
    let server = ServerThread::start(&ports);
    let mut agent = connected_agent(&ports);
    let mut supervisor = supervisor();
    assert!(agent.is_registered());
    assert_eq!(agent.session_state(), SessionRecoveryState::Registered);
    assert_eq!(server.registered_count(), 1);
    let kept = agent
        .get_embodiment()
        .export_device_registrations_as_config_json()
        .expect("export");

    end_session_like_a_genome_load(&server, &mut agent);
    assert_eq!(server.registered_count(), 0);
    assert!(!agent.is_registered());
    assert_eq!(agent.session_state(), SessionRecoveryState::Reconnecting);
    // Sends fail with the typed error, not an untyped connection string.
    assert!(matches!(
        agent.send_stored_sensor_data(),
        Err(FeagiAgentError::NotRegistered(_))
    ));

    let report = agent.supervised_tick_at(&mut supervisor, 0);
    assert!(
        report.rebuild_attempted && report.rebuild_succeeded,
        "{report:?}"
    );
    assert!(matches!(
        report.session_error,
        Some(FeagiAgentError::NotRegistered(_))
    ));
    assert!(agent.is_registered());
    assert_eq!(agent.session_state(), SessionRecoveryState::Registered);
    assert_eq!(report.state, SessionRecoveryState::Registered);
    assert_eq!(server.registered_count(), 1);

    // The new session carries the same device registrations, and sends work again.
    let by_agent = server.ask(|h| {
        h.get_device_registrations_by_agent_store()
            .values()
            .cloned()
            .collect::<Vec<_>>()
    });
    assert_eq!(by_agent, vec![kept]);
    agent
        .send_stored_sensor_data()
        .expect("send after recovery");

    // One rebuild per session end: more ticks do nothing.
    for i in 1..20 {
        let r = agent.supervised_tick_at(&mut supervisor, i * 10);
        assert!(!r.rebuild_attempted, "{r:?}");
    }
    assert_eq!(server.registered_count(), 1);
}

#[test]
fn fresh_device_registrations_replace_the_stale_descriptor_scoped_ones() {
    let ports = Ports::new();
    let server = ServerThread::start(&ports);
    let mut agent = connected_agent(&ports);
    let mut supervisor = supervisor();
    let descriptor = descriptor();

    let stale = {
        let d = descriptor.clone();
        server
            .ask(move |h| h.get_device_registrations_by_descriptor(&d).cloned())
            .expect("descriptor-scoped registrations stored")
    };
    end_session_like_a_genome_load(&server, &mut agent);
    // They survive session teardown on purpose.
    let d = descriptor.clone();
    assert_eq!(
        server.ask(move |h| h.get_device_registrations_by_descriptor(&d).cloned()),
        Some(stale.clone())
    );

    // The agent now declares one more device than the stale copy holds.
    register_infrared(&agent, 1, 2);
    let fresh = agent
        .get_embodiment()
        .export_device_registrations_as_config_json()
        .expect("export");
    assert_ne!(fresh, stale);

    let report = agent.supervised_tick_at(&mut supervisor, 0);
    assert!(report.rebuild_succeeded, "{report:?}");
    let d = descriptor.clone();
    assert_eq!(
        server.ask(move |h| h.get_device_registrations_by_descriptor(&d).cloned()),
        Some(fresh.clone()),
        "descriptor-scoped registrations were not replaced by the re-registering agent's"
    );
    let by_agent = server.ask(|h| {
        h.get_device_registrations_by_agent_store()
            .values()
            .cloned()
            .collect::<Vec<_>>()
    });
    assert_eq!(by_agent, vec![fresh]);
}

#[test]
fn server_gone_reaches_cannot_reach_then_retries_every_30s_and_recovers() {
    let ports = Ports::new();
    let mut server = ServerThread::start(&ports);
    let mut agent = connected_agent(&ports);
    let mut supervisor = supervisor();
    end_session_like_a_genome_load(&server, &mut agent);
    server.stop();

    let mut attempts: Vec<u64> = Vec::new();
    let mut states: Vec<SessionRecoveryState> = Vec::new();
    let mut now = 0u64;
    // 70 s of mocked time in 500 ms ticks; each failed attempt costs REBUILD_TIMEOUT_MS of real time.
    while now <= 70_000 {
        let r = agent.supervised_tick_at(&mut supervisor, now);
        if r.rebuild_attempted {
            assert!(!r.rebuild_succeeded);
            attempts.push(now);
        }
        if states.last() != Some(&r.state) {
            states.push(r.state);
        }
        assert_eq!(agent.session_state(), r.state);
        now += 500;
    }
    assert_eq!(
        states,
        vec![
            SessionRecoveryState::Reconnecting,
            SessionRecoveryState::CannotReach
        ]
    );
    // Three fast attempts one cooldown apart, then every 30 s from the last fast attempt.
    assert_eq!(
        attempts,
        vec![0, 1_000, 2_000, 32_000, 62_000],
        "{attempts:?}"
    );
    assert!(!agent.is_registered());

    // FEAGI comes back: the next slow attempt registers and the state clears.
    let server = ServerThread::start(&ports);
    let mut recovered = false;
    while !recovered && now <= 100_000 {
        let r = agent.supervised_tick_at(&mut supervisor, now);
        recovered = r.rebuild_succeeded;
        now += 500;
    }
    assert!(
        recovered,
        "agent never re-registered after the server came back"
    );
    assert_eq!(agent.session_state(), SessionRecoveryState::Registered);
    assert_eq!(server.registered_count(), 1);
    assert!(now >= 92_000, "slow retry came early: {now}");
}
