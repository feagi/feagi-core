use std::collections::HashMap;

use feagi_agent::clients::{
    SessionAction, SessionEvent, SessionInit, SessionPhase, SessionStateMachine,
    SessionTimingConfig,
};
use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken};
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};

fn make_init() -> SessionInit {
    let agent_descriptor =
        AgentDescriptor::new("neuraville", "test_agent", 1).expect("valid descriptor");
    let auth_token = AuthToken::new([0u8; 32]);
    SessionInit {
        agent_descriptor,
        auth_token,
        requested_capabilities: vec![
            AgentCapabilities::SendSensorData,
            AgentCapabilities::ReceiveMotorData,
        ],
        timing: SessionTimingConfig {
            heartbeat_interval_ms: 1000,
            registration_deadline_ms: None,
        },
    }
}

#[test]
fn start_connect_emits_control_connect_action() {
    let mut sm = SessionStateMachine::new(make_init());
    let actions = sm.start_connect(0);
    assert!(matches!(sm.phase(), SessionPhase::ControlConnecting));
    assert!(matches!(
        actions.as_slice(),
        [SessionAction::ControlRequestConnect]
    ));
}

#[test]
fn control_active_emits_registration_action() {
    let mut sm = SessionStateMachine::new(make_init());
    let _ = sm.start_connect(0);
    let actions = sm.step(
        1,
        &[SessionEvent::ControlObserved {
            state: FeagiEndpointState::ActiveWaiting,
            message: None,
        }],
    );
    assert!(matches!(sm.phase(), SessionPhase::Registering));
    assert!(actions
        .iter()
        .any(|a| matches!(a, SessionAction::ControlSendRegistration { .. })));
}

#[test]
fn registration_success_without_required_endpoints_fails() {
    let mut sm = SessionStateMachine::new(make_init());
    let _ = sm.start_connect(0);
    let _ = sm.step(
        1,
        &[SessionEvent::ControlObserved {
            state: FeagiEndpointState::ActiveWaiting,
            message: None,
        }],
    );

    // Success response but empty endpoints map should fail.
    use feagi_agent::command_and_control::agent_registration_message::{
        AgentRegistrationMessage, RegistrationResponse,
    };
    use feagi_agent::command_and_control::FeagiMessage;
    let empty: HashMap<AgentCapabilities, TransportProtocolEndpoint> = HashMap::new();
    let msg =
        FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(
            RegistrationResponse::Success(feagi_io::AgentID::new_blank(), empty),
        ));
    let _actions = sm.step(
        2,
        &[SessionEvent::ControlObserved {
            state: FeagiEndpointState::ActiveWaiting,
            message: Some(msg),
        }],
    );
    assert!(matches!(sm.phase(), SessionPhase::Failed));
    assert!(sm.last_error().is_some());
}
