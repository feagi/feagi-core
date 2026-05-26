use core::error::Error;
use feagi_logging_and_errors::{FeagiError, FeagiErrorKey, FeagiErrorKeyTrait, FeagiErrorTrait};

#[derive(FeagiErrorKey)]
struct MissingGenomeKey {
    context: &'static str,
    genome_id: u16,
    cortical_area: u8,
}

#[derive(FeagiError)]
enum GenomeError {
    MissingGenome(MissingGenomeKey),
}

#[derive(FeagiErrorKey)]
struct AgentDisconnectedKey {
    context: &'static str,
    agent_id: u32,
}

#[derive(FeagiError)]
enum RuntimeError {
    AgentDisconnected(AgentDisconnectedKey),
    Genome(GenomeError),
}

#[test]
fn error_key_derive_generates_constructor_and_context_access() {
    let key = MissingGenomeKey::new("missing genome", 7, 3);

    assert_eq!(key.context(), "missing genome");
    assert_eq!(FeagiErrorKeyTrait::context(&key), "missing genome");
    assert_eq!(key.to_string(), "missing genome");
    assert_eq!(key.genome_id, 7);
    assert_eq!(key.cortical_area, 3);
}

#[test]
fn error_enum_derive_wraps_error_keys() {
    let error = GenomeError::MissingGenome(MissingGenomeKey::new("missing genome", 7, 3));

    assert_eq!(error.context(), "missing genome");
    assert_eq!(FeagiErrorTrait::context(&error), "missing genome");
    assert_eq!(error.to_string(), "missing genome");
    assert!(error.source().is_some());
}

#[test]
fn error_enum_derive_wraps_nested_errors() {
    let error = RuntimeError::Genome(GenomeError::MissingGenome(MissingGenomeKey::new(
        "missing genome",
        7,
        3,
    )));

    assert_eq!(error.context(), "missing genome");
    assert_eq!(error.to_string(), "missing genome");
    assert!(matches!(
        error.source().and_then(Error::source),
        Some(source) if source.to_string() == "missing genome"
    ));
}

#[test]
fn error_enum_derive_wraps_multiple_key_types() {
    let key = AgentDisconnectedKey::new("agent disconnected", 42);
    assert_eq!(key.agent_id, 42);

    let error = RuntimeError::AgentDisconnected(key);

    assert_eq!(error.context(), "agent disconnected");
    assert_eq!(error.to_string(), "agent disconnected");
}
