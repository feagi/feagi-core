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
    connectome_container_version, connectome_envelope_manifest, load_connectome_from_bytes,
    save_connectome_to_bytes, ConnectomeError, Result, CURRENT_CONTAINER_VERSION,
};

const BRAIN_ARTIFACT_MANIFEST_TAG: &str = "brain_artifact_manifest";
/// Contract v2 records the embedded genome's blocking validator errors in the manifest.
/// Contract v1 manifests carry none and still import.
const CURRENT_CONTRACT_VERSION: u32 = 2;
const CURRENT_CONNECTOME_SCHEMA_VERSION: u32 = CONNECTOME_SNAPSHOT_SCHEMA_VERSION;

/// The independently versioned parts of a brain artifact.
///
/// Used by `ConnectomeError::UnsupportedArtifactVersion` to name which part of an artifact is
/// newer than this build supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactVersionComponent {
    /// Manifest contract version.
    Contract,
    /// Binary container format version.
    Container,
    /// Connectome snapshot schema version.
    ConnectomeSchema,
    /// Embedded genome schema version.
    GenomeSchema,
}

impl ArtifactVersionComponent {
    /// Stable snake_case name, identical to the serialized form.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::Container => "container",
            Self::ConnectomeSchema => "connectome_schema",
            Self::GenomeSchema => "genome_schema",
        }
    }
}

impl std::fmt::Display for ArtifactVersionComponent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Highest version of each artifact component that this build reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedArtifactVersions {
    pub contract: u32,
    pub container: u32,
    pub connectome_schema: u32,
    pub genome_schema: u32,
}

/// Versions this build supports, built from the same constants validation compares against.
pub fn supported_brain_artifact_versions() -> SupportedArtifactVersions {
    SupportedArtifactVersions {
        contract: CURRENT_CONTRACT_VERSION,
        container: CURRENT_CONTAINER_VERSION,
        connectome_schema: CURRENT_CONNECTOME_SCHEMA_VERSION,
        genome_schema: feagi_evolutionary::genome::CURRENT_SCHEMA_VERSION.as_u32(),
    }
}

/// Build the structured error for an artifact component newer than this build supports.
fn unsupported_version(
    component: ArtifactVersionComponent,
    artifact_version: u32,
    supported_version: u32,
) -> ConnectomeError {
    ConnectomeError::UnsupportedArtifactVersion {
        component,
        artifact_version,
        supported_version,
    }
}

/// Version fields of an artifact manifest, read without the rest of the manifest.
///
/// Unknown fields are ignored (no `deny_unknown_fields`) so a manifest written by a newer build,
/// which may carry more fields, still yields its versions.
#[derive(Debug, Deserialize)]
struct ManifestVersionHeader {
    contract_version: u32,
    connectome_schema_version: u32,
    #[serde(default)]
    genome_schema_version: Option<u32>,
}

/// Parse only the version fields of a manifest JSON string.
fn read_manifest_version_header(manifest_json: &str) -> Result<ManifestVersionHeader> {
    serde_json::from_str(manifest_json).map_err(|error| {
        ConnectomeError::BrainArtifact(format!("embedded manifest is invalid: {error}"))
    })
}

/// Reject an artifact whose container or envelope manifest declares a newer version than this
/// build supports, before the body is decoded (a newer body may not decode at all).
///
/// Components are checked in the order container, contract, connectome schema, genome schema.
/// Containers without an envelope manifest (v1, v2) are checked after decode instead.
fn reject_newer_header_versions(source_bytes: &[u8], container_version: u32) -> Result<()> {
    if container_version > CURRENT_CONTAINER_VERSION {
        return Err(unsupported_version(
            ArtifactVersionComponent::Container,
            container_version,
            CURRENT_CONTAINER_VERSION,
        ));
    }
    let Some(manifest_json) = connectome_envelope_manifest(source_bytes)? else {
        return Ok(());
    };
    let header = read_manifest_version_header(manifest_json)?;
    let supported = supported_brain_artifact_versions();
    if header.contract_version > supported.contract {
        return Err(unsupported_version(
            ArtifactVersionComponent::Contract,
            header.contract_version,
            supported.contract,
        ));
    }
    if header.connectome_schema_version > supported.connectome_schema {
        return Err(unsupported_version(
            ArtifactVersionComponent::ConnectomeSchema,
            header.connectome_schema_version,
            supported.connectome_schema,
        ));
    }
    if let Some(genome_version) = header.genome_schema_version {
        if genome_version > supported.genome_schema {
            return Err(unsupported_version(
                ArtifactVersionComponent::GenomeSchema,
                genome_version,
                supported.genome_schema,
            ));
        }
    }
    Ok(())
}

/// Reject an embedded genome newer than this build's genome schema, before migration.
///
/// A genome whose version cannot be detected is left to the migration step, which reports
/// it as invalid.
fn reject_newer_embedded_genome(genome_json: &str) -> Result<()> {
    let Ok(genome) = serde_json::from_str::<Value>(genome_json) else {
        return Ok(());
    };
    let Ok(detected) = feagi_evolutionary::genome::detect_schema_version(&genome) else {
        return Ok(());
    };
    let supported = feagi_evolutionary::genome::CURRENT_SCHEMA_VERSION.as_u32();
    if detected.as_u32() > supported {
        return Err(unsupported_version(
            ArtifactVersionComponent::GenomeSchema,
            detected.as_u32(),
            supported,
        ));
    }
    Ok(())
}

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
    /// Blocking validator errors of the embedded genome when the artifact was written. An
    /// artifact with errors is still saved and imports in degraded mode (`genome_validity`
    /// false). Absent in contract v1 manifests.
    #[serde(default)]
    pub blocking_errors: Vec<String>,
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
    /// Blocking validator errors of the embedded genome. The artifact stays importable in
    /// degraded mode; this is the list behind `genome_validity=false`.
    #[serde(default)]
    pub blocking_errors: Vec<String>,
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
/// The manifest is always written, including for a genome with blocking validator errors:
/// those errors are recorded in it so the brain can be saved and resumed in degraded mode.
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

    let mut blocking_errors = Vec::new();
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
            blocking_errors = report.blocking_errors.clone();
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
            blocking_errors,
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
    reject_newer_header_versions(source_bytes, source_container_version)?;
    let mut snapshot = load_connectome_from_bytes(source_bytes)?;
    snapshot.validate().map_err(|error| {
        ConnectomeError::BrainArtifact(format!("connectome snapshot validation failed: {error}"))
    })?;
    let source_connectome_schema_version = snapshot.version;

    // Reached for containers without an envelope manifest (v1, v2); v3 is rejected earlier.
    if source_connectome_schema_version > CURRENT_CONNECTOME_SCHEMA_VERSION {
        return Err(unsupported_version(
            ArtifactVersionComponent::ConnectomeSchema,
            source_connectome_schema_version,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
        ));
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
            return Err(unsupported_version(
                ArtifactVersionComponent::Contract,
                manifest.contract_version,
                CURRENT_CONTRACT_VERSION,
            ));
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
    let mut blocking_errors = Vec::new();

    match snapshot.genome_json.clone() {
        Some(genome_json) => {
            // The manifest digest covers the embedded genome exactly as stored at export,
            // so it is verified before migration or normalization can rewrite any value.
            if let Some(expected_digest) = previous_manifest
                .as_ref()
                .and_then(|manifest| manifest.genome_sha256.as_deref())
            {
                verify_stored_genome_digest(&genome_json, expected_digest)?;
            }

            // A manifest with a genome version was already checked from the header; without
            // one the embedded genome is the only source of its version.
            if previous_manifest
                .as_ref()
                .and_then(|manifest| manifest.genome_schema_version)
                .is_none()
            {
                reject_newer_embedded_genome(&genome_json)?;
            }

            let (migrated_genome, chain_report) =
                feagi_evolutionary::migrate_genome_json_to_current(&genome_json).map_err(
                    |error| {
                        ConnectomeError::BrainArtifact(format!(
                            "embedded genome migration failed: {error}"
                        ))
                    },
                )?;

            // Blocking errors do not reject the artifact: it resumes in degraded mode. Only a
            // genome that cannot produce a runtime genome (below) is a hard failure.
            blocking_errors = chain_report.blocking_errors.clone();
            if !blocking_errors.is_empty() {
                warnings.push(format!(
                    "embedded genome has {} blocking validator error(s); it will resume in degraded mode",
                    blocking_errors.len()
                ));
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
        blocking_errors: blocking_errors.clone(),
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
            blocking_errors,
        },
        manifest,
    })
}

/// Check the stored embedded genome against the manifest digest.
///
/// Export stores the canonical JSON of the genome and the sha256 of exactly those bytes, so
/// the stored genome is canonicalized the same way and hashed before any migration runs.
fn verify_stored_genome_digest(stored_genome_json: &str, expected_digest: &str) -> Result<()> {
    let stored_genome: Value = serde_json::from_str(stored_genome_json).map_err(|error| {
        ConnectomeError::BrainArtifact(format!("embedded genome is not valid JSON: {error}"))
    })?;
    let canonical_stored = canonical_json_string(&stored_genome)?;
    if sha256_hex(canonical_stored.as_bytes()) != expected_digest {
        return Err(ConnectomeError::BrainArtifact(
            "embedded genome digest does not match manifest".to_string(),
        ));
    }
    Ok(())
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
    use serde_json::json;

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
        assert_eq!(result.report.target_genome_schema_version, Some(7));
        assert!(result
            .report
            .migration_steps
            .contains(&"genome:v2_to_v3".to_string()));
        assert!(result
            .report
            .migration_steps
            .contains(&"genome:v3_to_v4".to_string()));
        assert!(result
            .report
            .migration_steps
            .contains(&"genome:v4_to_v5".to_string()));
        assert!(result
            .report
            .migration_steps
            .contains(&"genome:v5_to_v6".to_string()));
        assert_eq!(migrated_genome["genome_schema_version"], 7);
        assert!(migrated_snapshot
            .metadata
            .tags
            .contains_key(BRAIN_ARTIFACT_MANIFEST_TAG));
        assert_eq!(
            result.manifest.genome_sha256.as_deref().map(str::len),
            Some(64)
        );
    }

    /// A genome that loads but fails a blocking validator rule (leak is a fraction, 150 is not).
    fn genome_with_blocking_error() -> String {
        let mut genome: Value = serde_json::from_str(&both_membership_keys_genome())
            .expect("fixture genome is valid JSON");
        genome["blueprint"]["_power"]["leak_coefficient"] = json!(150);
        genome.to_string()
    }

    #[test]
    fn artifact_with_blocking_errors_is_saved_and_resumes_degraded() {
        let source = snapshot(
            ConnectomePersistMode::Lite,
            Some(genome_with_blocking_error()),
        );

        let bytes = save_connectome_to_bytes(&source)
            .expect("a genome with blocking errors must still be saved");
        let result = validate_and_migrate_brain_artifact(&bytes)
            .expect("a genome with blocking errors must still resume");

        assert!(result.report.valid && result.report.compatible);
        assert!(result
            .report
            .blocking_errors
            .iter()
            .any(|error| error.contains("leak_coefficient")));
        assert_eq!(result.manifest.contract_version, 2);
        assert_eq!(
            result.manifest.blocking_errors,
            result.report.blocking_errors
        );
        let saved = load_connectome_from_bytes(&bytes).expect("decode saved artifact");
        let saved_manifest = read_manifest(&saved)
            .expect("manifest reads")
            .expect("manifest present");
        assert!(!saved_manifest.blocking_errors.is_empty());
    }

    /// A current-version (v7) genome holding a legacy-percent leak that `V7Normalizer` repairs.
    fn v7_genome_with_repairable_leak() -> Value {
        let mut genome: Value = serde_json::from_str(&both_membership_keys_genome())
            .expect("fixture genome is valid JSON");
        genome["genome_schema_version"] = json!(7);
        genome["brain_regions"]["root"] = json!({
            "title": "Root",
            "cortical_areas": ["_power", "_death"]
        });
        genome["blueprint"]["_power"]["leak_coefficient"] = json!(50);
        genome
    }

    /// Build a lite artifact whose manifest digest covers `stored_genome` (canonical) as-is.
    fn artifact_with_digest_over(stored_genome: &Value, digest_genome: &Value) -> Vec<u8> {
        let mut source = snapshot(
            ConnectomePersistMode::Lite,
            Some(canonical_json_string(stored_genome).expect("canonicalize stored genome")),
        );
        let digest_over = canonical_json_string(digest_genome).expect("canonicalize digest genome");
        write_manifest(
            &mut source,
            &BrainArtifactManifest {
                contract_version: CURRENT_CONTRACT_VERSION,
                container_format_version: CURRENT_CONTAINER_VERSION,
                connectome_schema_version: CURRENT_CONNECTOME_SCHEMA_VERSION,
                genome_schema_version: Some(7),
                genome_sha256: Some(sha256_hex(digest_over.as_bytes())),
                producer_version: "0.0.1".to_string(),
                migration_steps: Vec::new(),
                blocking_errors: Vec::new(),
            },
        )
        .expect("write manifest");
        save_connectome_to_bytes(&source).expect("save artifact with a prebuilt manifest")
    }

    #[test]
    fn stored_genome_repaired_by_v7_normalizer_still_imports() {
        // Digest over the un-normalized stored genome, as written before V7Normalizer existed.
        let stored = v7_genome_with_repairable_leak();
        let bytes = artifact_with_digest_over(&stored, &stored);

        let result = validate_and_migrate_brain_artifact(&bytes)
            .expect("digest of the stored genome must verify before normalization");

        assert_eq!(result.report.source_genome_schema_version, Some(7));
        assert_eq!(result.report.target_genome_schema_version, Some(7));
        let migrated_snapshot =
            load_connectome_from_bytes(&result.artifact_bytes).expect("decode migrated artifact");
        let migrated_genome: Value = serde_json::from_str(
            migrated_snapshot
                .genome_json
                .as_deref()
                .expect("migrated genome"),
        )
        .expect("parse migrated genome");
        assert_eq!(
            migrated_genome["blueprint"]["_power"]["leak_coefficient"],
            json!(0.5)
        );
        // The rewritten manifest digests the normalized genome, so it re-imports cleanly.
        let migrated_digest = sha256_hex(
            canonical_json_string(&migrated_genome)
                .expect("canonicalize migrated genome")
                .as_bytes(),
        );
        assert_eq!(
            result.manifest.genome_sha256.as_deref(),
            Some(migrated_digest.as_str())
        );
        validate_and_migrate_brain_artifact(&result.artifact_bytes)
            .expect("normalized artifact re-imports");
    }

    #[test]
    fn tampered_stored_genome_is_rejected_by_digest() {
        let stored = v7_genome_with_repairable_leak();
        let mut tampered = stored.clone();
        tampered["genome_title"] = json!("tampered after the digest was written");
        let bytes = artifact_with_digest_over(&tampered, &stored);

        let error = validate_and_migrate_brain_artifact(&bytes)
            .expect_err("a genome that differs from the manifest digest must be rejected");

        assert!(error
            .to_string()
            .contains("embedded genome digest does not match manifest"));
    }

    #[test]
    fn artifact_without_blocking_errors_records_none() {
        let source = snapshot(
            ConnectomePersistMode::Lite,
            Some(both_membership_keys_genome()),
        );
        let bytes = save_connectome_to_bytes(&source).expect("save clean artifact");
        let result = validate_and_migrate_brain_artifact(&bytes).expect("clean artifact resumes");
        assert!(result.report.blocking_errors.is_empty());
        assert!(result.manifest.blocking_errors.is_empty());
    }

    #[test]
    fn contract_v1_manifest_still_imports() {
        let mut source = snapshot(ConnectomePersistMode::Full, None);
        // Contract v1 had no `blocking_errors` field.
        let v1_manifest = json!({
            "contract_version": 1,
            "container_format_version": CURRENT_CONTAINER_VERSION,
            "connectome_schema_version": CURRENT_CONNECTOME_SCHEMA_VERSION,
            "genome_schema_version": null,
            "genome_sha256": null,
            "producer_version": "0.0.1",
            "migration_steps": []
        });
        source.metadata.tags.insert(
            BRAIN_ARTIFACT_MANIFEST_TAG.to_string(),
            v1_manifest.to_string(),
        );
        let bytes = save_connectome_to_bytes(&source).expect("save with a v1 manifest");

        let result = validate_and_migrate_brain_artifact(&bytes).expect("v1 manifest imports");

        assert_eq!(result.manifest.contract_version, CURRENT_CONTRACT_VERSION);
        assert!(result.manifest.blocking_errors.is_empty());
    }

    #[test]
    fn artifact_with_unparseable_genome_still_fails() {
        let source = snapshot(ConnectomePersistMode::Lite, Some("{ not json".to_string()));
        let bytes = legacy_v2_bytes(&source);
        assert!(validate_and_migrate_brain_artifact(&bytes).is_err());
    }

    fn both_membership_keys_genome() -> String {
        r#"{
            "genome_id": "t-both",
            "genome_title": "t",
            "genome_description": "t",
            "version": "2.0",
            "blueprint": {
                "_power": {
                    "cortical_name": "Power",
                    "block_boundaries": [1, 1, 1],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "CORE"
                },
                "_death": {
                    "cortical_name": "Death",
                    "block_boundaries": [1, 1, 1],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "CORE"
                }
            },
            "brain_regions": {
                "root": {
                    "title": "Root",
                    "areas": ["_power"],
                    "cortical_areas": ["_death"]
                }
            },
            "neuron_morphologies": {},
            "physiology": { "simulation_timestep": 0.025, "max_age": 1 },
            "stats": { "innate_cortical_area_count": 2, "innate_neuron_count": 0, "innate_synapse_count": 0 },
            "signatures": { "genome": "0", "blueprint": "0", "physiology": "0" },
            "timestamp": 0.0
        }"#
        .to_string()
    }

    #[test]
    fn lite_artifact_folds_duplicate_region_membership_keys() {
        let source = snapshot(
            ConnectomePersistMode::Lite,
            Some(both_membership_keys_genome()),
        );
        let bytes = legacy_v2_bytes(&source);

        let result = validate_and_migrate_brain_artifact(&bytes).expect("migrate both-key genome");
        assert!(result.report.valid);
        assert_eq!(result.report.target_genome_schema_version, Some(7));

        let migrated_snapshot =
            load_connectome_from_bytes(&result.artifact_bytes).expect("decode migrated artifact");
        let migrated_genome: Value = serde_json::from_str(
            migrated_snapshot
                .genome_json
                .as_deref()
                .expect("migrated genome"),
        )
        .expect("parse migrated genome");
        let root = &migrated_genome["brain_regions"]["root"];
        assert!(root.get("areas").is_none());
        assert_eq!(root["cortical_areas"].as_array().unwrap().len(), 2);
        feagi_evolutionary::load_genome_with_report(
            migrated_snapshot.genome_json.as_deref().unwrap(),
        )
        .expect("folded genome parses");

        let again = validate_and_migrate_brain_artifact(&result.artifact_bytes)
            .expect("migrated artifact migrates again");
        assert!(again.report.valid);
        let again_snapshot =
            load_connectome_from_bytes(&again.artifact_bytes).expect("decode second artifact");
        let again_genome: Value = serde_json::from_str(
            again_snapshot
                .genome_json
                .as_deref()
                .expect("second genome"),
        )
        .expect("parse second genome");
        assert!(again_genome["brain_regions"]["root"].get("areas").is_none());
        assert_eq!(
            again_genome["brain_regions"]["root"]["cortical_areas"],
            root["cortical_areas"]
        );
    }

    #[test]
    fn lite_artifact_folds_areas_when_embedded_genome_is_already_current() {
        let mut genome: Value =
            serde_json::from_str(&both_membership_keys_genome()).expect("genome json");
        genome["genome_schema_version"] = json!(6);
        let source = snapshot(ConnectomePersistMode::Lite, Some(genome.to_string()));
        let bytes = legacy_v2_bytes(&source);

        let result =
            validate_and_migrate_brain_artifact(&bytes).expect("current genome with areas folds");
        assert!(result.report.valid);
        assert_eq!(result.report.source_genome_schema_version, Some(6));
        assert_eq!(result.report.target_genome_schema_version, Some(7));

        let migrated_snapshot =
            load_connectome_from_bytes(&result.artifact_bytes).expect("decode migrated artifact");
        let migrated_genome: Value = serde_json::from_str(
            migrated_snapshot
                .genome_json
                .as_deref()
                .expect("migrated genome"),
        )
        .expect("parse migrated genome");
        assert!(migrated_genome["brain_regions"]["root"]
            .get("areas")
            .is_none());
        assert_eq!(
            migrated_genome["brain_regions"]["root"]["cortical_areas"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn rejects_future_connectome_schema() {
        let mut future = snapshot(ConnectomePersistMode::Full, None);
        future.version = CURRENT_CONNECTOME_SCHEMA_VERSION + 1;
        let bytes = legacy_v2_bytes(&future);

        let error = validate_and_migrate_brain_artifact(&bytes)
            .expect_err("future schema must be rejected");

        assert!(matches!(
            error,
            ConnectomeError::UnsupportedArtifactVersion {
                component: ArtifactVersionComponent::ConnectomeSchema,
                artifact_version,
                supported_version,
            } if artifact_version == CURRENT_CONNECTOME_SCHEMA_VERSION + 1
                && supported_version == CURRENT_CONNECTOME_SCHEMA_VERSION
        ));
    }

    /// Container v3 artifact whose envelope manifest declares the given versions. The snapshot
    /// body carries the same connectome schema version, so the artifact is self-consistent.
    fn artifact_with_manifest_versions(
        genome_json: Option<String>,
        contract: u32,
        connectome_schema: u32,
        genome_schema: Option<u32>,
        extra_manifest_fields: Value,
    ) -> Vec<u8> {
        let mode = if genome_json.is_some() {
            ConnectomePersistMode::Lite
        } else {
            ConnectomePersistMode::Full
        };
        let mut source = snapshot(mode, genome_json);
        source.version = connectome_schema;
        let mut manifest = json!({
            "contract_version": contract,
            "container_format_version": CURRENT_CONTAINER_VERSION,
            "connectome_schema_version": connectome_schema,
            "genome_schema_version": genome_schema,
            "genome_sha256": null,
            "producer_version": "9.9.9",
            "migration_steps": [],
            "blocking_errors": []
        });
        manifest
            .as_object_mut()
            .expect("manifest is an object")
            .extend(
                extra_manifest_fields
                    .as_object()
                    .expect("extra fields are an object")
                    .clone(),
            );
        source.metadata.tags.insert(
            BRAIN_ARTIFACT_MANIFEST_TAG.to_string(),
            manifest.to_string(),
        );
        save_connectome_to_bytes(&source).expect("save artifact with a prebuilt manifest")
    }

    /// Flip the last body byte so that decoding the body fails its checksum. A version error
    /// from such bytes proves it was raised from the header, before the body was decoded.
    fn with_corrupt_body(mut bytes: Vec<u8>) -> Vec<u8> {
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        bytes
    }

    fn assert_unsupported(
        error: ConnectomeError,
        expected_component: ArtifactVersionComponent,
        expected_artifact: u32,
        expected_supported: u32,
    ) {
        match error {
            ConnectomeError::UnsupportedArtifactVersion {
                component,
                artifact_version,
                supported_version,
            } => {
                assert_eq!(component, expected_component);
                assert_eq!(artifact_version, expected_artifact);
                assert_eq!(supported_version, expected_supported);
            }
            other => panic!("expected UnsupportedArtifactVersion, got {other:?}"),
        }
    }

    #[test]
    fn newer_container_version_is_structured_error() {
        let mut bytes = save_connectome_to_bytes(&snapshot(ConnectomePersistMode::Full, None))
            .expect("serialize test snapshot");
        let newer = CURRENT_CONTAINER_VERSION + 1;
        bytes[5..9].copy_from_slice(&newer.to_le_bytes());

        let error = validate_and_migrate_brain_artifact(&bytes).expect_err("newer container");

        assert_unsupported(
            error,
            ArtifactVersionComponent::Container,
            newer,
            CURRENT_CONTAINER_VERSION,
        );
    }

    #[test]
    fn newer_contract_version_is_rejected_from_header_before_body_decode() {
        let bytes = with_corrupt_body(artifact_with_manifest_versions(
            None,
            CURRENT_CONTRACT_VERSION + 1,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
            None,
            json!({}),
        ));

        let error = validate_and_migrate_brain_artifact(&bytes).expect_err("newer contract");

        assert_unsupported(
            error,
            ArtifactVersionComponent::Contract,
            CURRENT_CONTRACT_VERSION + 1,
            CURRENT_CONTRACT_VERSION,
        );
    }

    #[test]
    fn newer_connectome_schema_in_v3_is_rejected_from_header_before_body_decode() {
        let newer = CURRENT_CONNECTOME_SCHEMA_VERSION + 1;
        let bytes = with_corrupt_body(artifact_with_manifest_versions(
            None,
            CURRENT_CONTRACT_VERSION,
            newer,
            None,
            json!({}),
        ));

        let error = validate_and_migrate_brain_artifact(&bytes).expect_err("newer schema");

        assert_unsupported(
            error,
            ArtifactVersionComponent::ConnectomeSchema,
            newer,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
        );
    }

    #[test]
    fn newer_genome_schema_in_manifest_is_rejected_from_header_before_body_decode() {
        let supported = supported_brain_artifact_versions().genome_schema;
        let bytes = with_corrupt_body(artifact_with_manifest_versions(
            None,
            CURRENT_CONTRACT_VERSION,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
            Some(supported + 1),
            json!({}),
        ));

        let error = validate_and_migrate_brain_artifact(&bytes).expect_err("newer genome");

        assert_unsupported(
            error,
            ArtifactVersionComponent::GenomeSchema,
            supported + 1,
            supported,
        );
    }

    #[test]
    fn first_newer_component_in_order_is_reported() {
        let versions = supported_brain_artifact_versions();
        let bytes = artifact_with_manifest_versions(
            None,
            versions.contract + 1,
            versions.connectome_schema + 1,
            Some(versions.genome_schema + 1),
            json!({}),
        );

        let error = validate_and_migrate_brain_artifact(&bytes).expect_err("all newer");

        assert_unsupported(
            error,
            ArtifactVersionComponent::Contract,
            versions.contract + 1,
            versions.contract,
        );
    }

    #[test]
    fn newer_embedded_genome_without_manifest_genome_version_is_structured_error() {
        let supported = supported_brain_artifact_versions().genome_schema;
        let mut genome: Value =
            serde_json::from_str(&both_membership_keys_genome()).expect("genome json");
        genome["genome_schema_version"] = json!(supported + 1);

        // Legacy container: no envelope manifest at all.
        let legacy = legacy_v2_bytes(&snapshot(
            ConnectomePersistMode::Lite,
            Some(genome.to_string()),
        ));
        let error = validate_and_migrate_brain_artifact(&legacy).expect_err("newer genome");
        assert_unsupported(
            error,
            ArtifactVersionComponent::GenomeSchema,
            supported + 1,
            supported,
        );

        // Container v3: manifest present but without a genome version.
        let v3 = artifact_with_manifest_versions(
            Some(genome.to_string()),
            CURRENT_CONTRACT_VERSION,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
            None,
            json!({}),
        );
        let error = validate_and_migrate_brain_artifact(&v3).expect_err("newer genome");
        assert_unsupported(
            error,
            ArtifactVersionComponent::GenomeSchema,
            supported + 1,
            supported,
        );
    }

    #[test]
    fn corrupt_artifacts_are_not_reported_as_newer_versions() {
        let current = save_connectome_to_bytes(&snapshot(ConnectomePersistMode::Full, None))
            .expect("serialize test snapshot");

        let corrupt_body = validate_and_migrate_brain_artifact(&with_corrupt_body(current.clone()))
            .expect_err("corrupt body");
        assert!(matches!(corrupt_body, ConnectomeError::ChecksumMismatch));

        let bad_magic =
            validate_and_migrate_brain_artifact(b"NOTFEAGI-----").expect_err("bad magic");
        assert!(matches!(bad_magic, ConnectomeError::InvalidMagic(_)));

        let truncated =
            validate_and_migrate_brain_artifact(&current[..12]).expect_err("truncated header");
        assert!(matches!(truncated, ConnectomeError::BrainArtifact(_)));

        let unparseable_manifest = {
            let mut source = snapshot(ConnectomePersistMode::Full, None);
            source.metadata.tags.insert(
                BRAIN_ARTIFACT_MANIFEST_TAG.to_string(),
                "{ not a manifest".to_string(),
            );
            save_connectome_to_bytes(&source).expect("save with unparseable manifest")
        };
        let error = validate_and_migrate_brain_artifact(&unparseable_manifest)
            .expect_err("invalid manifest");
        assert!(matches!(error, ConnectomeError::BrainArtifact(_)));
        assert!(error.to_string().contains("embedded manifest is invalid"));
    }

    #[test]
    fn current_artifact_still_passes() {
        let bytes = artifact_with_manifest_versions(
            None,
            CURRENT_CONTRACT_VERSION,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
            None,
            json!({}),
        );

        let result = validate_and_migrate_brain_artifact(&bytes).expect("current artifact");

        assert!(result.report.valid && result.report.compatible);
    }

    #[test]
    fn supported_versions_match_constants() {
        let versions = supported_brain_artifact_versions();

        assert_eq!(versions.contract, CURRENT_CONTRACT_VERSION);
        assert_eq!(versions.container, CURRENT_CONTAINER_VERSION);
        assert_eq!(
            versions.connectome_schema,
            CURRENT_CONNECTOME_SCHEMA_VERSION
        );
        assert_eq!(
            versions.genome_schema,
            feagi_evolutionary::genome::CURRENT_SCHEMA_VERSION.as_u32()
        );
    }

    #[test]
    fn version_component_names_are_stable() {
        let expected = [
            (ArtifactVersionComponent::Contract, "contract"),
            (ArtifactVersionComponent::Container, "container"),
            (
                ArtifactVersionComponent::ConnectomeSchema,
                "connectome_schema",
            ),
            (ArtifactVersionComponent::GenomeSchema, "genome_schema"),
        ];
        for (component, name) in expected {
            assert_eq!(component.as_str(), name);
            assert_eq!(component.to_string(), name);
            assert_eq!(
                serde_json::to_value(component).expect("serialize component"),
                json!(name)
            );
        }
    }

    #[test]
    fn unsupported_version_error_message_is_exact() {
        let error = ConnectomeError::UnsupportedArtifactVersion {
            component: ArtifactVersionComponent::GenomeSchema,
            artifact_version: 8,
            supported_version: 7,
        };

        assert_eq!(
            error.to_string(),
            "Brain artifact genome_schema v8 is newer than supported v7"
        );
    }

    #[test]
    fn header_reader_ignores_unknown_manifest_fields() {
        let header = read_manifest_version_header(
            &json!({
                "contract_version": 5,
                "connectome_schema_version": 6,
                "genome_schema_version": 7,
                "added_by_a_newer_build": { "nested": [1, 2, 3] },
                "another_field": "value"
            })
            .to_string(),
        )
        .expect("unknown fields are ignored");

        assert_eq!(header.contract_version, 5);
        assert_eq!(header.connectome_schema_version, 6);
        assert_eq!(header.genome_schema_version, Some(7));

        let without_genome = read_manifest_version_header(
            &json!({ "contract_version": 1, "connectome_schema_version": 1 }).to_string(),
        )
        .expect("genome version is optional");
        assert_eq!(without_genome.genome_schema_version, None);
    }

    #[test]
    fn newer_manifest_with_extra_fields_still_reports_its_version() {
        let bytes = artifact_with_manifest_versions(
            None,
            CURRENT_CONTRACT_VERSION + 1,
            CURRENT_CONNECTOME_SCHEMA_VERSION,
            None,
            json!({ "field_from_a_newer_contract": { "a": 1 } }),
        );

        let error = validate_and_migrate_brain_artifact(&bytes).expect_err("newer contract");

        assert_unsupported(
            error,
            ArtifactVersionComponent::Contract,
            CURRENT_CONTRACT_VERSION + 1,
            CURRENT_CONTRACT_VERSION,
        );
    }
}
