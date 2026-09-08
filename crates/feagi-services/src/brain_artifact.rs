// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Versioned validation and migration for genome-dependent connectome artifacts.
//!
//! The migration is pure: callers provide source bytes and receive new bytes plus
//! a report. The source artifact is never modified.

use std::collections::{BTreeMap, BTreeSet};

use feagi_npu_neural::types::connectome::{
    ConnectomePersistMode, ConnectomeSnapshot, SerializableNeuronReference,
    CONNECTOME_SNAPSHOT_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::connectome::{
    connectome_container_version, load_connectome_from_bytes, save_connectome_to_bytes,
    ConnectomeError, Result,
};

const BRAIN_ARTIFACT_MANIFEST_TAG: &str = "brain_artifact_manifest";
const CURRENT_CONTRACT_VERSION: u32 = 1;
const CURRENT_CONNECTOME_SCHEMA_VERSION: u32 = CONNECTOME_SNAPSHOT_SCHEMA_VERSION;
const CURRENT_CONTAINER_VERSION: u32 = 3;

/// Compatibility metadata embedded inside the connectome's metadata tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrainArtifactManifest {
    pub contract_version: u32,
    pub container_format_version: u32,
    pub connectome_schema_version: u32,
    pub genome_schema_version: Option<u32>,
    pub genome_sha256: Option<String>,
    pub producer_version: String,
    pub migration_steps: Vec<String>,
}

/// Deterministic result of validating and migrating a brain artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainArtifactMigrationReport {
    pub valid: bool,
    pub compatible: bool,
    pub source_container_version: u32,
    pub target_container_version: u32,
    pub source_connectome_schema_version: u32,
    pub target_connectome_schema_version: u32,
    pub source_genome_schema_version: Option<u32>,
    pub target_genome_schema_version: Option<u32>,
    pub genome_sha256: Option<String>,
    pub migration_steps: Vec<String>,
    pub identifier_remaps: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}

/// New artifact bytes and their validation/migration report.
#[derive(Debug, Clone)]
pub struct BrainArtifactMigrationResult {
    pub artifact_bytes: Vec<u8>,
    pub report: BrainArtifactMigrationReport,
    pub manifest: BrainArtifactManifest,
}

/// Embed a manifest when a newly exported snapshot already carries a current genome.
///
/// Historical snapshots are intentionally left unstamped so the migration path
/// can preserve their true source versions.
pub(crate) fn embed_manifest_for_current_export(snapshot: &mut ConnectomeSnapshot) -> Result<()> {
    if snapshot
        .metadata
        .tags
        .contains_key(BRAIN_ARTIFACT_MANIFEST_TAG)
    {
        return Ok(());
    }
    if snapshot.version != CURRENT_CONNECTOME_SCHEMA_VERSION {
        return Ok(());
    }

    let (genome_schema_version, genome_sha256) = match snapshot.genome_json.as_deref() {
        Some(genome_json) => {
            let (current_genome, report) = feagi_evolutionary::migrate_genome_json_to_current(
                genome_json,
            )
            .map_err(|error| {
                ConnectomeError::BrainArtifact(format!(
                    "exported embedded genome is invalid: {error}"
                ))
            })?;
            if report.from_version != report.to_version || !report.is_blocking_clean() {
                return Ok(());
            }
            let canonical_genome = canonical_json_string(&current_genome)?;
            snapshot.genome_json = Some(canonical_genome.clone());
            (
                Some(report.to_version.as_u32()),
                Some(sha256_hex(canonical_genome.as_bytes())),
            )
        }
        None if snapshot.persist_mode == ConnectomePersistMode::Lite => return Ok(()),
        None => (None, None),
    };

    write_manifest(
        snapshot,
        &BrainArtifactManifest {
            contract_version: CURRENT_CONTRACT_VERSION,
            container_format_version: CURRENT_CONTAINER_VERSION,
            connectome_schema_version: CURRENT_CONNECTOME_SCHEMA_VERSION,
            genome_schema_version,
            genome_sha256,
            producer_version: env!("CARGO_PKG_VERSION").to_string(),
            migration_steps: Vec::new(),
        },
    )
}

pub(crate) fn encoded_manifest(snapshot: &ConnectomeSnapshot) -> Result<&str> {
    snapshot
        .metadata
        .tags
        .get(BRAIN_ARTIFACT_MANIFEST_TAG)
        .map(String::as_str)
        .ok_or_else(|| {
            ConnectomeError::BrainArtifact(
                "current connectome container requires an artifact manifest".to_string(),
            )
        })
}

/// Validate an artifact and migrate its embedded genome and semantic references.
///
/// Lite artifacts require an embedded genome. Full artifacts migrate one when
/// present, but remain valid without it because they carry complete structure.
pub fn validate_and_migrate_brain_artifact(
    source_bytes: &[u8],
) -> Result<BrainArtifactMigrationResult> {
    let source_container_version = connectome_container_version(source_bytes)?;
    let mut snapshot = load_connectome_from_bytes(source_bytes)?;
    snapshot.validate().map_err(|error| {
        ConnectomeError::BrainArtifact(format!("connectome snapshot validation failed: {error}"))
    })?;
    let source_connectome_schema_version = snapshot.version;

    if source_connectome_schema_version > CURRENT_CONNECTOME_SCHEMA_VERSION {
        return Err(ConnectomeError::BrainArtifact(format!(
            "connectome schema v{} is newer than supported v{}",
            source_connectome_schema_version, CURRENT_CONNECTOME_SCHEMA_VERSION
        )));
    }
    if source_connectome_schema_version < CURRENT_CONNECTOME_SCHEMA_VERSION {
        return Err(ConnectomeError::BrainArtifact(format!(
            "no migration is registered from connectome schema v{} to v{}",
            source_connectome_schema_version, CURRENT_CONNECTOME_SCHEMA_VERSION
        )));
    }

    let previous_manifest = read_manifest(&snapshot)?;
    if let Some(manifest) = &previous_manifest {
        if manifest.contract_version > CURRENT_CONTRACT_VERSION {
            return Err(ConnectomeError::BrainArtifact(format!(
                "artifact contract v{} is newer than supported v{}",
                manifest.contract_version, CURRENT_CONTRACT_VERSION
            )));
        }
        if manifest.container_format_version != source_container_version {
            return Err(ConnectomeError::BrainArtifact(
                "manifest container version does not match binary envelope".to_string(),
            ));
        }
        if manifest.connectome_schema_version != source_connectome_schema_version {
            return Err(ConnectomeError::BrainArtifact(
                "manifest connectome version does not match serialized snapshot".to_string(),
            ));
        }
    }
    let mut migration_steps = previous_manifest
        .as_ref()
        .map(|manifest| manifest.migration_steps.clone())
        .unwrap_or_default();
    let mut identifier_remaps = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut source_genome_schema_version = None;
    let mut target_genome_schema_version = None;
    let mut genome_sha256 = None;

    match snapshot.genome_json.clone() {
        Some(genome_json) => {
            let (migrated_genome, chain_report) =
                feagi_evolutionary::migrate_genome_json_to_current(&genome_json).map_err(
                    |error| {
                        ConnectomeError::BrainArtifact(format!(
                            "embedded genome migration failed: {error}"
                        ))
                    },
                )?;

            if !chain_report.is_blocking_clean() {
                return Err(ConnectomeError::BrainArtifact(format!(
                    "embedded genome validation failed: {}",
                    chain_report.blocking_errors.join("; ")
                )));
            }

            let migrated_genome_json =
                serde_json::to_string(&migrated_genome).map_err(|error| {
                    ConnectomeError::BrainArtifact(format!(
                        "failed to serialize migrated embedded genome: {error}"
                    ))
                })?;
            feagi_evolutionary::load_genome_with_report(&migrated_genome_json).map_err(
                |error| {
                    ConnectomeError::BrainArtifact(format!(
                        "migrated embedded genome cannot produce a runtime genome: {error}"
                    ))
                },
            )?;

            source_genome_schema_version = Some(chain_report.from_version.as_u32());
            target_genome_schema_version = Some(chain_report.to_version.as_u32());
            if let Some(manifest_version) = previous_manifest
                .as_ref()
                .and_then(|manifest| manifest.genome_schema_version)
            {
                if manifest_version != chain_report.from_version.as_u32() {
                    return Err(ConnectomeError::BrainArtifact(
                        "manifest genome version does not match embedded genome".to_string(),
                    ));
                }
            }
            for step in &chain_report.per_step_diagnostics {
                identifier_remaps.extend(step.identifier_remaps.clone());
            }
            migration_steps.extend(
                chain_report
                    .migrators_applied
                    .iter()
                    .map(|name| format!("genome:{name}")),
            );
            warnings.extend(chain_report.advisory_warnings);

            apply_identifier_remaps(&mut snapshot, &identifier_remaps);
            validate_snapshot_references(&snapshot, &migrated_genome)?;

            let canonical_genome = canonical_json_string(&migrated_genome)?;
            genome_sha256 = Some(sha256_hex(canonical_genome.as_bytes()));
            if chain_report.from_version == chain_report.to_version {
                if let Some(expected_digest) = previous_manifest
                    .as_ref()
                    .and_then(|manifest| manifest.genome_sha256.as_deref())
                {
                    if genome_sha256.as_deref() != Some(expected_digest) {
                        return Err(ConnectomeError::BrainArtifact(
                            "embedded genome digest does not match manifest".to_string(),
                        ));
                    }
                }
            }
            snapshot.genome_json = Some(canonical_genome);
            snapshot.validate().map_err(|error| {
                ConnectomeError::BrainArtifact(format!(
                    "migrated connectome snapshot validation failed: {error}"
                ))
            })?;
        }
        None if snapshot.persist_mode == ConnectomePersistMode::Lite => {
            return Err(ConnectomeError::BrainArtifact(
                "connectome-lite artifact is missing its embedded genome".to_string(),
            ));
        }
        None => {
            warnings.push(
                "full connectome has no embedded genome; genome compatibility was not evaluated"
                    .to_string(),
            );
        }
    }

    if source_container_version < CURRENT_CONTAINER_VERSION {
        migration_steps.push(format!(
            "container:v{source_container_version}->v{CURRENT_CONTAINER_VERSION}"
        ));
    }

    let manifest = BrainArtifactManifest {
        contract_version: CURRENT_CONTRACT_VERSION,
        container_format_version: CURRENT_CONTAINER_VERSION,
        connectome_schema_version: CURRENT_CONNECTOME_SCHEMA_VERSION,
        genome_schema_version: target_genome_schema_version,
        genome_sha256: genome_sha256.clone(),
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        migration_steps: migration_steps.clone(),
    };
    write_manifest(&mut snapshot, &manifest)?;

    let artifact_bytes = save_connectome_to_bytes(&snapshot)?;
    Ok(BrainArtifactMigrationResult {
        artifact_bytes,
        report: BrainArtifactMigrationReport {
            valid: true,
            compatible: true,
            source_container_version,
            target_container_version: CURRENT_CONTAINER_VERSION,
            source_connectome_schema_version,
            target_connectome_schema_version: CURRENT_CONNECTOME_SCHEMA_VERSION,
            source_genome_schema_version,
            target_genome_schema_version,
            genome_sha256,
            migration_steps,
            identifier_remaps,
            warnings,
        },
        manifest,
    })
}

fn read_manifest(snapshot: &ConnectomeSnapshot) -> Result<Option<BrainArtifactManifest>> {
    let Some(raw_manifest) = snapshot.metadata.tags.get(BRAIN_ARTIFACT_MANIFEST_TAG) else {
        return Ok(None);
    };
    let manifest = serde_json::from_str(raw_manifest).map_err(|error| {
        ConnectomeError::BrainArtifact(format!("embedded manifest is invalid: {error}"))
    })?;
    Ok(Some(manifest))
}

fn write_manifest(
    snapshot: &mut ConnectomeSnapshot,
    manifest: &BrainArtifactManifest,
) -> Result<()> {
    let encoded = serde_json::to_string(manifest).map_err(|error| {
        ConnectomeError::BrainArtifact(format!("failed to encode artifact manifest: {error}"))
    })?;
    snapshot
        .metadata
        .tags
        .insert(BRAIN_ARTIFACT_MANIFEST_TAG.to_string(), encoded);
    Ok(())
}

fn apply_identifier_remaps(
    snapshot: &mut ConnectomeSnapshot,
    identifier_remaps: &BTreeMap<String, String>,
) {
    let remap = |id: &mut String| {
        if let Some(replacement) = identifier_remaps.get(id) {
            *id = replacement.clone();
        }
    };

    for id in &mut snapshot.memory_area_ids {
        remap(id);
    }
    for (source, destination) in &mut snapshot.plastic_mappings {
        remap(source);
        remap(destination);
    }
    for neuron in &mut snapshot.long_term_memory_neurons {
        if let Some(cortical_id) = &mut neuron.cortical_id {
            remap(cortical_id);
        }
    }
    for (_, frames) in &mut snapshot.long_term_memory_replay_frames {
        for frame in frames {
            if let Some(cortical_id) = &mut frame.upstream_cortical_id {
                remap(cortical_id);
            }
        }
    }
    for synapse in &mut snapshot.lite_synapses {
        remap_neuron_reference(&mut synapse.source, identifier_remaps);
        remap_neuron_reference(&mut synapse.target, identifier_remaps);
    }
}

fn remap_neuron_reference(
    reference: &mut SerializableNeuronReference,
    identifier_remaps: &BTreeMap<String, String>,
) {
    let cortical_id = match reference {
        SerializableNeuronReference::Regular { cortical_id, .. }
        | SerializableNeuronReference::LongTermMemory { cortical_id, .. } => cortical_id,
    };
    if let Some(replacement) = identifier_remaps.get(cortical_id) {
        *cortical_id = replacement.clone();
    }
}

fn validate_snapshot_references(snapshot: &ConnectomeSnapshot, genome: &Value) -> Result<()> {
    let valid_ids: BTreeSet<&str> = genome
        .get("blueprint")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            ConnectomeError::BrainArtifact(
                "migrated embedded genome has no blueprint object".to_string(),
            )
        })?
        .keys()
        .map(String::as_str)
        .collect();

    let mut referenced_ids = BTreeSet::new();
    referenced_ids.extend(snapshot.memory_area_ids.iter().map(String::as_str));
    for (source, destination) in &snapshot.plastic_mappings {
        referenced_ids.insert(source);
        referenced_ids.insert(destination);
    }
    for neuron in &snapshot.long_term_memory_neurons {
        if let Some(id) = neuron.cortical_id.as_deref() {
            referenced_ids.insert(id);
        }
    }
    for (_, frames) in &snapshot.long_term_memory_replay_frames {
        for frame in frames {
            if let Some(id) = frame.upstream_cortical_id.as_deref() {
                referenced_ids.insert(id);
            }
        }
    }
    for synapse in &snapshot.lite_synapses {
        referenced_ids.insert(neuron_reference_cortical_id(&synapse.source));
        referenced_ids.insert(neuron_reference_cortical_id(&synapse.target));
    }

    let missing: Vec<&str> = referenced_ids.difference(&valid_ids).copied().collect();
    if !missing.is_empty() {
        return Err(ConnectomeError::BrainArtifact(format!(
            "connectome references cortical IDs absent from migrated genome: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn neuron_reference_cortical_id(reference: &SerializableNeuronReference) -> &str {
    match reference {
        SerializableNeuronReference::Regular { cortical_id, .. }
        | SerializableNeuronReference::LongTermMemory { cortical_id, .. } => cortical_id,
    }
}

fn canonical_json_string(value: &Value) -> Result<String> {
    fn canonicalize(value: &Value) -> Value {
        match value {
            Value::Object(object) => {
                let sorted: BTreeMap<&String, &Value> = object.iter().collect();
                Value::Object(
                    sorted
                        .into_iter()
                        .map(|(key, value)| (key.clone(), canonicalize(value)))
                        .collect(),
                )
            }
            Value::Array(items) => Value::Array(items.iter().map(canonicalize).collect()),
            scalar => scalar.clone(),
        }
    }

    serde_json::to_string(&canonicalize(value)).map_err(|error| {
        ConnectomeError::BrainArtifact(format!("failed to canonicalize embedded genome: {error}"))
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use ahash::AHashMap;
    use feagi_npu_neural::types::connectome::{
        ConnectomeMetadata, SerializableNeuronArray, SerializableSynapseArray,
        LITE_EDGE_ENCODING_SEMANTIC_V1, LITE_EDGE_ENCODING_TAG,
    };

    use super::*;

    fn snapshot(mode: ConnectomePersistMode, genome_json: Option<String>) -> ConnectomeSnapshot {
        let mut metadata = ConnectomeMetadata::default();
        if mode == ConnectomePersistMode::Lite {
            metadata.tags.insert(
                LITE_EDGE_ENCODING_TAG.to_string(),
                LITE_EDGE_ENCODING_SEMANTIC_V1.to_string(),
            );
        }
        ConnectomeSnapshot {
            version: CURRENT_CONNECTOME_SCHEMA_VERSION,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray::default(),
            cortical_area_names: AHashMap::new(),
            burst_count: 0,
            power_amount: 1.0,
            fire_ledger_window: 1,
            metadata,
            persist_mode: mode,
            genome_json,
            memory_area_ids: Vec::new(),
            plastic_mappings: Vec::new(),
            brain_region_ids: Vec::new(),
            long_term_memory_neurons: Vec::new(),
            long_term_memory_replay_frames: Vec::new(),
            lite_synapses: Vec::new(),
        }
    }

    fn legacy_v2_bytes(snapshot: &ConnectomeSnapshot) -> Vec<u8> {
        let body = bincode::serialize(snapshot).expect("serialize legacy snapshot");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"FEAGI");
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&super::super::connectome::calculate_checksum(&body).to_le_bytes());
        bytes.extend_from_slice(&body);
        bytes
    }

    #[test]
    fn lite_artifact_requires_embedded_genome() {
        let bytes = legacy_v2_bytes(&snapshot(ConnectomePersistMode::Lite, None));

        let error = validate_and_migrate_brain_artifact(&bytes)
            .expect_err("lite artifact without genome must fail");

        assert!(error.to_string().contains("genome"));
    }

    #[test]
    fn full_artifact_without_genome_is_valid_with_warning() {
        let bytes = save_connectome_to_bytes(&snapshot(ConnectomePersistMode::Full, None))
            .expect("serialize test snapshot");

        let result = validate_and_migrate_brain_artifact(&bytes).expect("validate full artifact");

        assert!(result.report.valid);
        assert!(result.report.compatible);
        assert_eq!(
            result.report.target_container_version,
            CURRENT_CONTAINER_VERSION
        );
        assert_eq!(result.report.warnings.len(), 1);
        assert!(result.manifest.genome_schema_version.is_none());
    }

    #[test]
    fn lite_artifact_migrates_embedded_genome_before_persisting_manifest() {
        let source = snapshot(
            ConnectomePersistMode::Lite,
            Some(feagi_evolutionary::BAREBONES_GENOME_JSON.to_string()),
        );
        let bytes = legacy_v2_bytes(&source);

        let result = validate_and_migrate_brain_artifact(&bytes).expect("migrate lite artifact");
        let migrated_snapshot =
            load_connectome_from_bytes(&result.artifact_bytes).expect("decode migrated artifact");
        let migrated_genome: Value = serde_json::from_str(
            migrated_snapshot
                .genome_json
                .as_deref()
                .expect("migrated genome"),
        )
        .expect("parse migrated genome");

        assert_eq!(result.report.source_genome_schema_version, Some(2));
        assert_eq!(result.report.target_genome_schema_version, Some(3));
        assert!(result
            .report
            .migration_steps
            .contains(&"genome:v2_to_v3".to_string()));
        assert_eq!(migrated_genome["genome_schema_version"], 3);
        assert!(migrated_snapshot
            .metadata
            .tags
            .contains_key(BRAIN_ARTIFACT_MANIFEST_TAG));
        assert_eq!(
            result.manifest.genome_sha256.as_deref().map(str::len),
            Some(64)
        );
    }

    #[test]
    fn rejects_future_connectome_schema() {
        let mut future = snapshot(ConnectomePersistMode::Full, None);
        future.version = CURRENT_CONNECTOME_SCHEMA_VERSION + 1;
        let bytes = legacy_v2_bytes(&future);

        let error = validate_and_migrate_brain_artifact(&bytes)
            .expect_err("future schema must be rejected");

        assert!(error.to_string().contains("newer than supported"));
    }
}
