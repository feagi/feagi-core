//! Liveness reporting and stale-agent pruning.
//!
//! Registration does not keep an agent connected. [`FeagiAgentHandler`] deregisters any agent
//! that stays quiet past `heartbeat_timeout`, and that teardown recycles the agent's publishers,
//! which closes its sockets. These tests pin both halves of that contract: the pruning itself,
//! and the liveness snapshot that lets callers see an agent approaching the threshold instead of
//! only observing the disconnect afterwards.

#![cfg(feature = "agent-server")]

use std::thread::sleep;
use std::time::Duration;

use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
use feagi_agent::{AgentCapabilities, AgentDescriptor};
use feagi_io::AgentID;

/// A handler with no transport servers attached. `poll_command_and_control` still runs the stale
/// scan in this state, so pruning is testable without binding sockets.
fn handler_with_timeout(heartbeat_timeout: Duration) -> FeagiAgentHandler {
    FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig {
            heartbeat_timeout,
            stale_check_interval: Duration::from_millis(1),
        },
    )
}

fn descriptor(agent_name: &str) -> AgentDescriptor {
    AgentDescriptor::new("neuraville", agent_name, 1).expect("descriptor inputs must be valid")
}

#[test]
fn liveness_is_empty_when_no_agents_are_registered() {
    let handler = handler_with_timeout(Duration::from_secs(30));

    assert!(handler.get_agent_liveness().is_empty());
}

#[test]
fn liveness_reports_the_configured_timeout_and_scan_interval() {
    let handler = handler_with_timeout(Duration::from_secs(30));

    let config = handler.get_liveness_config();
    assert_eq!(config.heartbeat_timeout, Duration::from_secs(30));
    assert_eq!(config.stale_check_interval, Duration::from_millis(1));
}

#[test]
fn liveness_reports_descriptor_and_capabilities_for_a_registered_agent() {
    let mut handler = handler_with_timeout(Duration::from_secs(30));
    let agent_id = AgentID::new_random();
    handler.register_logical_agent(
        agent_id,
        descriptor("brain-visualizer"),
        vec![AgentCapabilities::ReceiveNeuronVisualizations],
    );

    let records = handler.get_agent_liveness();

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.agent_id, agent_id);
    assert_eq!(record.descriptor.agent_name(), "brain-visualizer");
    assert_eq!(record.descriptor.manufacturer(), "neuraville");
    assert_eq!(record.descriptor.agent_version(), 1);
    assert_eq!(
        record.capabilities,
        vec![AgentCapabilities::ReceiveNeuronVisualizations]
    );
    // Registration seeds both activity clocks, so a just-registered agent reports ages rather
    // than the `None` that signals missing internal state.
    assert!(record.last_activity_age.is_some());
    assert!(record.last_command_control_age.is_some());
}

#[test]
fn liveness_ages_grow_while_an_agent_stays_quiet() {
    let mut handler = handler_with_timeout(Duration::from_secs(30));
    handler.register_logical_agent(
        AgentID::new_random(),
        descriptor("quiet-agent"),
        vec![AgentCapabilities::ReceiveNeuronVisualizations],
    );

    let quiet_period = Duration::from_millis(60);
    sleep(quiet_period);
    let record = handler.get_agent_liveness().remove(0);

    // Both clocks advance because nothing refreshed either one. An agent held alive purely by
    // FEAGI's outbound traffic is distinguishable precisely because only the command/control
    // clock keeps climbing.
    assert!(record.last_activity_age.expect("activity age recorded") >= quiet_period);
    assert!(
        record
            .last_command_control_age
            .expect("command/control age recorded")
            >= quiet_period
    );
}

#[test]
fn liveness_records_are_ordered_by_agent_id() {
    let mut handler = handler_with_timeout(Duration::from_secs(30));
    for index in 0..4 {
        handler.register_logical_agent(
            AgentID::new_random(),
            descriptor(&format!("agent-{index}")),
            vec![AgentCapabilities::ReceiveNeuronVisualizations],
        );
    }

    let ordering: Vec<String> = handler
        .get_agent_liveness()
        .iter()
        .map(|record| record.agent_id.to_base64())
        .collect();

    let mut expected = ordering.clone();
    expected.sort();
    assert_eq!(ordering, expected);
}

#[test]
fn a_quiet_agent_is_pruned_once_the_heartbeat_timeout_elapses() {
    let heartbeat_timeout = Duration::from_millis(60);
    let mut handler = handler_with_timeout(heartbeat_timeout);
    let agent_id = AgentID::new_random();
    handler.register_logical_agent(
        agent_id,
        descriptor("brain-visualizer"),
        vec![AgentCapabilities::ReceiveNeuronVisualizations],
    );

    sleep(heartbeat_timeout * 2);
    handler
        .poll_command_and_control()
        .expect("polling with no transport servers must succeed");

    assert!(
        handler.get_all_registered_agents().is_empty(),
        "quiet agent must be deregistered"
    );
    assert!(
        handler.get_agent_liveness().is_empty(),
        "liveness must drop pruned agents"
    );
}

#[test]
fn an_agent_within_the_heartbeat_timeout_survives_the_stale_scan() {
    let mut handler = handler_with_timeout(Duration::from_secs(30));
    let agent_id = AgentID::new_random();
    handler.register_logical_agent(
        agent_id,
        descriptor("brain-visualizer"),
        vec![AgentCapabilities::ReceiveNeuronVisualizations],
    );

    sleep(Duration::from_millis(20));
    handler
        .poll_command_and_control()
        .expect("polling with no transport servers must succeed");

    assert_eq!(handler.get_agent_liveness().len(), 1);
    assert!(handler.get_all_registered_agents().contains_key(&agent_id));
}

#[test]
fn liveness_drops_agents_removed_by_forced_deregistration() {
    let mut handler = handler_with_timeout(Duration::from_secs(30));
    handler.register_logical_agent(
        AgentID::new_random(),
        descriptor("brain-visualizer"),
        vec![AgentCapabilities::ReceiveNeuronVisualizations],
    );

    let removed = handler.force_deregister_all_agents("test teardown");

    assert_eq!(removed.len(), 1);
    assert!(handler.get_agent_liveness().is_empty());
}
