//! Morphologies: named connection patterns (vectors, patterns, functions, composite) used by
//! cortical mappings. Converted from the old `morphology` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;
use feagi_cns_structures::identifiers::cortical_areas::Base64EncodedCorticalID;

/// A morphology definition, as listed by `Morphologies`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MorphologyDefinition {
    /// Morphology name
    pub name: String,
    /// Morphology type (vectors, patterns, functions, composite)
    #[serde(rename = "type")]
    pub morphology_type: String,
    /// Morphology class (e.g. custom)
    pub class: String,
    /// Type specific parameters
    pub parameters: serde_json::Value,
    /// Where the definition came from (always "genome")
    pub source: String,
}

define_request_message_category! {
    category_name: "Morphology",
    base_path: "v1/morphology",
    category_description: "Named connection patterns used by cortical mappings",
    read: {
        "MorphologyList": {
            path: "morphology_list",
            description: "Get all morphology names in alphabetical order",
            alloc_only: true,
            response: {
                morphology_list: Vec<String>,
            },
        },
        "MorphologyTypes": {
            path: "morphology_types",
            description: "Get the available morphology types",
            alloc_only: true,
            response: {
                types: Vec<String>,
            },
        },
        "ListTypes": {
            path: "list/types",
            description: "Get morphology names grouped by type",
            alloc_only: true,
            response: {
                types: HashMap<String, Vec<String>>, "Morphology names, keyed by type",
            },
        },
        "Morphologies": {
            path: "morphologies",
            description: "Get all morphology definitions with their complete configurations",
            alloc_only: true,
            response: {
                morphologies: HashMap<String, MorphologyDefinition>, "Definitions, keyed by name",
            },
        },
        "Info": {
            path: "info/:morphology_id<String>",
            path_item_descriptions: {
                morphology_id: "Morphology name",
            },
            description: "Get detailed properties of a morphology",
            alloc_only: true,
            response: {
                morphology_name: String,
                r#type: String, "Morphology type",
                class: String,
                parameters: serde_json::Value, "Type specific parameters",
                source: String, "Always genome",
            },
            error: {
                MorphologyNotFound: "No morphology has that name",
            },
        },
    },
    create: {
        "Morphology": {
            path: "morphology",
            description: "Create a new morphology",
            alloc_only: true,
            payload: {
                morphology_name: String, "Name of the new morphology",
                morphology_type: String, "vectors, patterns, functions or composite",
                morphology_parameters: serde_json::Value, "Type specific parameters",
            },
            response: {
                status: String,
            },
            error: {
                UnknownMorphologyType: "morphology_type is not a known type",
                InvalidParameters: "morphology_parameters do not match the type",
            },
        },
        "MorphologyProperties": {
            path: "morphology_properties",
            description: "Get detailed properties of a morphology",
            alloc_only: true,
            payload: {
                morphology_name: String,
            },
            response: {
                morphology_name: String,
                r#type: String, "Morphology type",
                class: String,
                parameters: serde_json::Value, "Type specific parameters",
                source: String, "Always genome",
            },
            error: {
                MorphologyNotFound: "No morphology has that name",
            },
        },
        "MorphologyUsage": {
            path: "morphology_usage",
            description: "Get every [source, destination] cortical area pair whose mapping uses a morphology",
            alloc_only: true,
            payload: {
                morphology_name: String,
            },
            response: {
                usage: Vec<[Base64EncodedCorticalID; 2]>, "[source area ID, destination area ID] pairs",
            },
        },
    },
    edit: {
        "Morphology": {
            path: "morphology",
            description: "Update an existing morphology",
            alloc_only: true,
            payload: {
                morphology_name: String, "Name of the morphology to update",
                morphology_type: String, "vectors, patterns, functions or composite",
                morphology_parameters: serde_json::Value, "Type specific parameters",
            },
            response: {
                status: String,
            },
            error: {
                MorphologyNotFound: "No morphology has that name",
                UnknownMorphologyType: "morphology_type is not a known type",
                InvalidParameters: "morphology_parameters do not match the type",
            },
        },
        "Rename": {
            path: "rename",
            description: "Rename a morphology and update all references to it in the genome",
            alloc_only: true,
            payload: {
                old_morphology_id: String, "Current name (alias: old_morphology_name)",
                new_morphology_id: String, "New name (alias: new_morphology_name)",
            },
            response: {
                status: String,
                old_morphology_id: String,
                new_morphology_id: String,
            },
            error: {
                MorphologyNotFound: "No morphology has the old name",
                MorphologyIdTaken: "A morphology with the new name already exists",
            },
        },
    },
    delete: {
        "Morphology": {
            path: "morphology",
            description: "Delete a morphology by name",
            alloc_only: true,
            payload: {
                morphology_name: String,
            },
            response: {
                status: String,
            },
            error: {
                MorphologyNotFound: "No morphology has that name",
            },
        },
        "Delete": {
            path: "delete/:morphology_id<String>",
            path_item_descriptions: {
                morphology_id: "Morphology name",
            },
            description: "Delete a morphology using path parameter",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
}

// NOTES:
// MorphologyTypes, List: old responses were bare arrays; wrapped in `types` / `morphologies`.
// ListTypes, Morphologies: old responses were bare maps; wrapped in `types` / `morphologies`. ListTypes was a stub returning {}.
// morphology_parameters / parameters left as serde_json::Value; shape depends on morphology_type.
// MorphologyProperties, MorphologyUsage are reads but live in `create` because the old endpoints were POST.
// MorphologyUsage: old response was a bare Vec<Vec<String>>; wrapped in `usage` and typed as pairs.
// Composite parameters may be wrapped as {"composite": {...}} (old code accepted both forms).
// POST create: unimplemented stub, payload unknown; response only had `message`.
// PUT update: unimplemented stub, payload unknown; response only had `message`.
// DELETE delete/{morphology_id}: unimplemented stub; response only had `message`.
