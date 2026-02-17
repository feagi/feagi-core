use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};

fn build_handler() -> FeagiAgentHandler {
    FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    )
}

#[test]
fn handler_creates_and_has_no_registered_agents() {
    let handler = build_handler();
    assert!(handler.get_all_registered_agents().is_empty());
}
