use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
use feagi_io::AgentID;

fn build_handler() -> FeagiAgentHandler {
    FeagiAgentHandler::new_with_liveness_config(Box::new(DummyAuth {}), AgentLivenessConfig::default())
}

#[test]
fn handler_creates_with_default_liveness_config() {
    let handler = build_handler();
    assert!(handler.get_all_registered_agents().is_empty());
}

#[test]
fn unregistered_session_ids_not_in_registered_agents() {
    let handler = build_handler();
    let session_id = AgentID::new_random();
    assert!(
        !handler.get_all_registered_agents().contains_key(&session_id),
        "Handler should not contain unregistered session IDs"
    );
}
