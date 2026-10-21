//! Genome: loading, saving, inspecting and amalgamating (merging) genomes.
//!
//! Converted from the old `genome` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// Summary of an amalgamation that is waiting for a destination
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmalgamationPendingSummary {
    /// ID of the pending amalgamation
    pub amalgamation_id: String,
    /// Title of the genome being amalgamated
    pub genome_title: String,
    /// Dimensions of the imported circuit bounding box (x, y, z)
    pub circuit_size: [i32; 3],
}

/// One entry of the amalgamation history
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmalgamationHistoryEntry {
    pub amalgamation_id: String,
    pub genome_title: String,
    /// Dimensions of the imported circuit bounding box (x, y, z)
    pub circuit_size: [i32; 3],
    /// "pending" | "confirmed" | "cancelled" | "replaced"
    pub status: String,
    /// Unix time in milliseconds
    pub timestamp_ms: i64,
}

/// Brain region as listed after an amalgamation is confirmed
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AmalgamationBrainRegion {
    pub region_id: String,
    pub title: String,
    pub description: String,
    pub parent_region_id: Option<String>,
    pub coordinate_2d: Vec<i32>,
    pub coordinate_3d: Vec<i32>,
    /// Cortical area IDs in this region
    pub areas: Vec<String>,
    /// Child region IDs
    pub regions: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub designated_inputs: Vec<String>,
    pub designated_outputs: Vec<String>,
}

/// An IO data type supported by a cortical template subunit
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalTemplateDataType {
    /// e.g. "Boolean", "Percentage", "Percentage2D", "CartesianPlane", "Misc"
    pub variant: String,
    /// "Absolute" | "Incremental"
    pub frame_change_handling: String,
    /// "Linear" | "Fractional", absent for non-percentage variants
    pub percentage_positioning: Option<String>,
    /// Raw IO configuration flag bitmask
    pub config_value: u16,
}

/// One subunit (cortical area) of a cortical template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalTemplateSubunit {
    pub relative_position: [i32; 3],
    pub channel_dimensions_default: [u32; 3],
    pub channel_dimensions_min: [u32; 3],
    pub channel_dimensions_max: [u32; 3],
    pub supported_data_types: Vec<CorticalTemplateDataType>,
}

/// A sensory or motor cortical unit template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalTemplate {
    /// "sensory" | "motor"
    pub r#type: String,
    pub friendly_name: String,
    pub cortical_id_prefix: String,
    pub number_of_cortical_areas: u32,
    /// Keyed by subunit index as a string
    pub subunits: HashMap<String, CorticalTemplateSubunit>,
    pub description: String,
}

define_request_message_category! {
    category_name: "Genome",
    base_path: "v1/genome",
    category_description: "Loading, saving, inspecting and amalgamating genomes",
    read: {
        "FileName": {
            path: "file_name",
            description: "Get the current genome file name",
            alloc_only: true,
            response: {
                genome_file_name: String,
            },
        },
        "Circuits": {
            path: "circuits",
            description: "Get the list of available circuits from the circuit library",
            alloc_only: true,
            response: {
                circuits: Vec<String>,
            },
        },
        "Name": {
            path: "name",
            description: "Get the current genome name",
            alloc_only: true,
            response: {
                name: String, "Genome name",
            },
        },
        "Timestamp": {
            path: "timestamp",
            description: "Get the genome creation or modification timestamp",
            response: {
                timestamp: i64,
            },
        },
        "Download": {
            path: "download",
            description: "Download the current genome as a JSON document (with the current runtime simulation timestep injected)",
            alloc_only: true,
            response: {
                genome: serde_json::Value, "Full genome JSON",
            },
        },
        "Properties": {
            path: "properties",
            description: "Get genome properties including metadata, size, and configuration details",
            alloc_only: true,
            response: {
                properties: HashMap<String, serde_json::Value>,
            },
        },
        "Metadata": {
            path: "metadata",
            description: "Get genome metadata (alternative endpoint to properties)",
            alloc_only: true,
            response: {
                properties: HashMap<String, serde_json::Value>,
            },
        },
        "Diff": {
            path: "diff",
            description: "Get a diff comparison between two genomes",
            alloc_only: true,
            payload: {
                genome_a: String, "First genome name",
                genome_b: String, "Second genome name",
            },
            response: {
                differences: Vec<serde_json::Value>,
            },
        },
        "Amalgamation": {
            path: "amalgamation",
            description: "Get the currently pending amalgamation, if any",
            alloc_only: true,
            response: {
                pending: Option<AmalgamationPendingSummary>, "None if no amalgamation is pending",
            },
        },
        "AmalgamationHistory": {
            path: "amalgamation_history",
            description: "Get the history of all genome amalgamation operations",
            alloc_only: true,
            response: {
                history: Vec<AmalgamationHistoryEntry>,
            },
        },
        "CorticalTemplate": {
            path: "cortical_template",
            description: "Get metadata about all sensory and motor cortical unit types, including per-subunit supported data types",
            alloc_only: true,
            response: {
                templates: HashMap<String, CorticalTemplate>, "Keyed by 'i'/'o' + cortical ID prefix",
            },
        },
        "DefaultsFiles": {
            path: "defaults/files",
            description: "Get the list of embedded default genomes (barebones, essential, test, vision)",
            alloc_only: true,
            response: {
                files: Vec<String>,
            },
        },
        "DownloadRegion": {
            path: "download_region",
            description: "Download a specific brain region of the genome as a genome JSON document",
            alloc_only: true,
            payload: {
                region_id: String, "ID of the brain region to export",
            },
            response: {
                genome: serde_json::Value, "Region genome JSON",
            },
        },
        "GenomeNumber": {
            path: "genome_number",
            description: "Get the current genome number or generation identifier",
            response: {
                genome_number: i32,
            },
        },
    },
    create: {
        "AmalgamationDestination": {
            path: "amalgamation_destination",
            description: "Confirm the pending amalgamation by placing it into a new brain region at the given origin",
            alloc_only: true,
            payload: {
                amalgamation_id: String, "Must match the pending amalgamation",
                circuit_origin_x: i32,
                circuit_origin_y: i32,
                circuit_origin_z: i32,
                rewire_mode: Option<String>, "Defaults to 'rewire_all'",
                brain_region_id: String, "Parent brain region for the new region",
            },
            response: {
                message: String,
                brain_regions: Vec<AmalgamationBrainRegion>, "All brain regions after the merge",
                skipped_existing_areas: Vec<String>, "Guest areas skipped because the host already has them",
                imported_new_area_count: u64,
                guest_cortical_area_count: u64,
                guest_custom_memory_id_remap_count: u64,
                imported_cortical_mappings: u64,
                skipped_cortical_mappings: u64,
                imported_morphology_count: u64,
                skipped_morphology_existing_count: u64,
                post_merge_brain_region_count: u64,
                post_merge_cortical_area_total: u64,
            },
            error: {
                NoAmalgamationPending: "No amalgamation is pending",
                AmalgamationIdMismatch: "The amalgamation ID does not match the pending amalgamation",
            },
        },
        "UploadBarebones": {
            path: "upload/barebones",
            description: "Load the minimal barebones genome",
            alloc_only: true,
            response: {
                success: bool,
                message: String,
                cortical_area_count: u64,
                brain_region_count: u64,
                genome_id: String,
                genome_title: String,
            },
        },
        "UploadEssential": {
            path: "upload/essential",
            description: "Load the essential genome with core sensory and motor areas",
            alloc_only: true,
            response: {
                success: bool,
                message: String,
                cortical_area_count: u64,
                brain_region_count: u64,
                genome_id: String,
                genome_title: String,
            },
        },
        "Save": {
            path: "save",
            description: "Save the current genome to a file",
            alloc_only: true,
            payload: {
                genome_id: Option<String>, "ID to assign to the saved genome",
                genome_title: Option<String>, "Title to assign to the saved genome",
                file_path: Option<String>, "Defaults to <data root>/cache/.genome/saved_genome_<timestamp>.json",
            },
            response: {
                message: String,
                file_path: String, "Where the genome was written",
            },
        },
        "Load": {
            path: "load",
            description: "Load a genome by name",
            alloc_only: true,
            payload: {
                genome_name: String,
            },
            response: {
                message: String,
                genome_title: String,
            },
        },
        "Upload": {
            path: "upload",
            description: "Upload and load a genome from a JSON payload",
            alloc_only: true,
            payload: {
                genome: serde_json::Value, "Full genome JSON",
            },
            response: {
                success: bool,
                message: String,
                cortical_area_count: u64,
                brain_region_count: u64,
            },
        },
        "Validate": {
            path: "validate",
            description: "Validate a genome structure for correctness and completeness",
            alloc_only: true,
            payload: {
                genome: serde_json::Value, "Full genome JSON",
            },
            response: {
                valid: bool,
                errors: Vec<String>,
                warnings: Vec<String>,
            },
        },
        "Transform": {
            path: "transform",
            description: "Transform a genome between formats (flat to hierarchical or vice versa)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "Clone": {
            path: "clone",
            description: "Clone the current genome with a new name, creating an independent copy",
            alloc_only: true,
            payload: {
                name: String, "Name of the copy",
            },
            response: {
                message: String,
            },
        },
        "Reset": {
            path: "reset",
            description: "Reset the genome to its default state, clearing all cortical areas and brain regions",
            alloc_only: true,
            response: {
                message: String,
            },
            error: {
                TransitionInProgress: "Another genome transition is in progress",
            },
        },
        "Merge": {
            path: "merge",
            description: "Merge another genome into the current genome",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "ExportFormat": {
            path: "export_format",
            description: "Export the genome in a specific format (JSON, YAML, binary, etc.)",
            alloc_only: true,
            payload: {
                format: String, "Target format",
            },
            response: {
                message: String,
            },
        },
        "AmalgamationByFilename": {
            path: "amalgamation_by_filename",
            description: "Queue an amalgamation of an embedded default genome by name",
            alloc_only: true,
            payload: {
                file_name: String, "One of barebones, essential, test, vision",
            },
            response: {
                message: String,
                amalgamation_id: String,
            },
            error: {
                UnsupportedFileName: "Not an embedded genome; use AmalgamationByPayload instead",
            },
        },
        "AmalgamationByPayload": {
            path: "amalgamation_by_payload",
            description: "Queue an amalgamation of a genome given as JSON",
            alloc_only: true,
            payload: {
                genome: serde_json::Value, "Full genome JSON",
            },
            response: {
                message: String,
                amalgamation_id: String,
            },
        },
        "AmalgamationByUpload": {
            path: "amalgamation_by_upload",
            description: "Queue an amalgamation of an uploaded genome file",
            alloc_only: true,
            payload: {
                file: String, "Genome file contents (UTF-8 JSON)",
            },
            response: {
                message: String,
                amalgamation_id: String,
            },
        },
        "AppendFile": {
            path: "append-file",
            description: "Append structures to the genome from a file",
            alloc_only: true,
            payload: {
                file: String, "Genome file contents (UTF-8 JSON)",
            },
            response: {
                message: String,
            },
        },
        "UploadFile": {
            path: "upload/file",
            description: "Upload and load a genome from a file",
            alloc_only: true,
            payload: {
                file: String, "Genome file contents (UTF-8 JSON)",
            },
            response: {
                success: bool,
                message: String,
                cortical_area_count: u64,
                brain_region_count: u64,
            },
        },
        "UploadFileEdit": {
            path: "upload/file/edit",
            description: "Upload a genome file with edit mode enabled",
            alloc_only: true,
            payload: {
                file: String, "Genome file contents (UTF-8 JSON)",
            },
            response: {
                message: String,
            },
        },
        "UploadString": {
            path: "upload/string",
            description: "Upload and load a genome from a JSON string",
            alloc_only: true,
            payload: {
                genome_json: String, "Genome as a JSON string",
            },
            response: {
                message: String,
            },
        },
    },
    delete: {
        "AmalgamationCancellation": {
            path: "amalgamation_cancellation",
            description: "Cancel the pending genome amalgamation",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
}

// NOTES:
// - Wrapped bare responses: Circuits (Vec), Name (String), Timestamp (i64), GenomeNumber (i32), DefaultsFiles (Vec), AmalgamationHistory (Vec), CorticalTemplate (map).
// - Download / DownloadRegion returned the genome JSON directly; wrapped as `genome: serde_json::Value`.
// - Upload / Validate / AmalgamationByPayload took the whole genome JSON as body; wrapped as `genome: serde_json::Value`.
// - UploadString took a bare JSON string body; wrapped as `genome_json`.
// - Properties / Metadata were stubs returning an empty untyped map; wrapped as `properties: HashMap<String, Value>`.
// - AmalgamationDestination mixed query params (origin, amalgamation_id, rewire_mode) and body (brain_region_id); merged into one payload.
// - AmalgamationBrainRegion coordinate and inputs/outputs types guessed (old code built them from untyped region properties).
// - AmalgamationByFilename also accepted `filename` / `genome_file_name` as aliases for `file_name`.
// - Multipart uploads (AmalgamationByUpload, AppendFile, UploadFile, UploadFileEdit) modeled as `file: String` contents.
// - Transform / Merge payloads were untyped maps never read (stubs); payload omitted.
// - Clone `name` and ExportFormat `format` payload fields are guessed (old code took an unread HashMap<String, String>).
// - Diff returned `differences: []` stub; element type unknown, kept as serde_json::Value.
// - CorticalTemplate counts/config_value types inferred from the genomic context crate (usize -> u32, bitmask u16).
// - Skipped POST /v1/feagi/genome/append: outside the genome base path and unimplemented (always errored).
