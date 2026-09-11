// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Encoding boundary for external `.genome` artifacts.
//!
//! Genome artifact encoding is independent from genome schema versioning. The
//! codec converts bytes to and from a schema-bearing [`serde_json::Value`];
//! migration and validation remain the responsibility of the genome chain.

use crate::{types::EvoError, EvoResult};
use serde_json::Value;

/// Required extension for external genome artifacts.
pub const GENOME_ARTIFACT_EXTENSION: &str = "genome";

/// Media type for the current JSON genome artifact encoding.
pub const GENOME_ARTIFACT_MEDIA_TYPE: &str = "application/vnd.feagi.genome+json";

/// Supported external artifact encoding.
///
/// This enum identifies representation only. It does not version genome
/// schemas and must never be used to dispatch schema migration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenomeArtifactEncoding {
    /// UTF-8 JSON containing a versioned genome document.
    Json,
}

/// Codec contract for external genome artifact bytes.
pub trait GenomeArtifactCodec: Send + Sync {
    /// Return the representation handled by this codec.
    fn encoding(&self) -> GenomeArtifactEncoding;

    /// Return the media type emitted for this representation.
    fn media_type(&self) -> &'static str;

    /// Decode artifact bytes into a schema-bearing genome document.
    fn decode(&self, artifact: &[u8]) -> EvoResult<Value>;

    /// Encode a schema-bearing genome document as artifact bytes.
    fn encode(&self, genome: &Value) -> EvoResult<Vec<u8>>;
}

/// Current UTF-8 JSON artifact codec.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonGenomeArtifactCodec;

impl GenomeArtifactCodec for JsonGenomeArtifactCodec {
    fn encoding(&self) -> GenomeArtifactEncoding {
        GenomeArtifactEncoding::Json
    }

    fn media_type(&self) -> &'static str {
        GENOME_ARTIFACT_MEDIA_TYPE
    }

    fn decode(&self, artifact: &[u8]) -> EvoResult<Value> {
        serde_json::from_slice(artifact)
            .map_err(|error| EvoError::invalid_genome(format!("Failed to parse JSON: {error}")))
    }

    fn encode(&self, genome: &Value) -> EvoResult<Vec<u8>> {
        serde_json::to_vec(genome).map_err(EvoError::from)
    }
}

/// Decode bytes using the current explicitly selected artifact encoding.
pub fn decode_genome_artifact(artifact: &[u8]) -> EvoResult<Value> {
    JsonGenomeArtifactCodec.decode(artifact)
}

/// Encode a genome using the current explicitly selected artifact encoding.
pub fn encode_genome_artifact(genome: &Value) -> EvoResult<Vec<u8>> {
    JsonGenomeArtifactCodec.encode(genome)
}

/// Validate that an external artifact filename uses `.genome`.
pub fn validate_genome_artifact_file_name(file_name: &str) -> EvoResult<()> {
    let leaf_name = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    let valid_extension = leaf_name.rsplit_once('.').is_some_and(|(stem, extension)| {
        !stem.is_empty() && extension.eq_ignore_ascii_case(GENOME_ARTIFACT_EXTENSION)
    });

    if valid_extension {
        Ok(())
    } else {
        Err(EvoError::invalid_genome(
            "Genome files must use the .genome extension",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_codec_round_trips_without_changing_schema_version() {
        let genome = json!({
            "genome_schema_version": 3,
            "version": "3.0",
            "blueprint": {}
        });

        let encoded = encode_genome_artifact(&genome).expect("JSON encoding should succeed");
        let decoded = decode_genome_artifact(&encoded).expect("JSON decoding should succeed");

        assert_eq!(decoded, genome);
        assert_eq!(decoded["genome_schema_version"], 3);
    }

    #[test]
    fn json_codec_rejects_malformed_artifact_bytes() {
        let result = decode_genome_artifact(b"not-json");

        assert!(result.is_err());
    }

    #[test]
    fn artifact_filename_contract_is_platform_independent() {
        assert!(validate_genome_artifact_file_name("brain.genome").is_ok());
        assert!(validate_genome_artifact_file_name("brain.GENOME").is_ok());
        assert!(validate_genome_artifact_file_name(r"C:\brains\brain.genome").is_ok());
        assert!(validate_genome_artifact_file_name("brain.json").is_err());
        assert!(validate_genome_artifact_file_name(".genome").is_err());
    }

    #[test]
    fn artifact_decode_precedes_existing_schema_migration_chain() {
        let artifact = br#"{
            "genome_id": "artifact-test",
            "genome_title": "Artifact test",
            "genome_description": "Codec and schema integration",
            "version": "2.0",
            "blueprint": {},
            "brain_regions": {},
            "neuron_morphologies": {},
            "physiology": {"simulation_timestep": 0.025, "max_age": 1},
            "stats": {
                "innate_cortical_area_count": 0,
                "innate_neuron_count": 0,
                "innate_synapse_count": 0
            },
            "signatures": {"genome": "0", "blueprint": "0", "physiology": "0"},
            "timestamp": 0.0
        }"#;

        let decoded = decode_genome_artifact(artifact).expect("artifact should decode");
        let (migrated, report) = crate::genome::migrate_genome_value_to_current(decoded)
            .expect("existing schema chain should migrate decoded genome");

        assert_eq!(report.from_version.as_u32(), 2);
        assert_eq!(report.to_version.as_u32(), 3);
        assert_eq!(migrated["genome_schema_version"], 3);
    }
}
