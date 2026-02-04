use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ahash::AHashMap;
use feagi_agent::server::auth::DummyAuth;
use feagi_agent::server::FeagiAgentHandler;
use feagi_config::FeagiConfig;
use feagi_evolutionary::templates::load_barebones_genome;
use feagi_npu_neural::types::connectome::{
    ConnectomeMetadata, ConnectomeSnapshot, SerializableNeuronArray, SerializableSynapseArray,
};

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let filename = format!("{name}-{timestamp}--temp.{extension}");
    PathBuf::from("/tmp").join(filename)
}

fn build_handler() -> FeagiAgentHandler {
    let config = FeagiConfig::default();
    FeagiAgentHandler::new_with_config(Box::new(DummyAuth {}), config)
}

#[test]
fn handler_saves_connectome_snapshot() {
    let handler = build_handler();
    let path = temp_path("feagi_connectome", "connectome");

    let snapshot = ConnectomeSnapshot {
        version: 2,
        neurons: SerializableNeuronArray::new(0),
        synapses: SerializableSynapseArray::default(),
        cortical_area_names: AHashMap::new(),
        burst_count: 0,
        power_amount: 0.0,
        fire_ledger_window: 0,
        metadata: ConnectomeMetadata::default(),
    };

    handler
        .save_connectome_snapshot(&snapshot, &path)
        .expect("Expected connectome save to succeed");

    assert!(path.exists(), "Connectome file should be created");
    fs::remove_file(&path).expect("Failed to remove connectome temp file");
}

#[test]
fn handler_saves_genome() {
    let handler = build_handler();
    let path = temp_path("feagi_genome", "json");

    let genome = load_barebones_genome().expect("Expected barebones genome to load");

    handler
        .save_genome(&genome, &path)
        .expect("Expected genome save to succeed");

    let metadata = fs::metadata(&path).expect("Genome file should exist");
    assert!(metadata.len() > 0, "Genome file should not be empty");
    fs::remove_file(&path).expect("Failed to remove genome temp file");
}
