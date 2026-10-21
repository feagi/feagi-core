//! Neuroplasticity: plasticity queue and per cortical area plasticity control.
//! Converted from the old `neuroplasticity` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

define_request_message_category! {
    category_name: "Neuroplasticity",
    base_path: "v1/neuroplasticity",
    category_description: "Neuroplasticity status, queue depth and per cortical area plasticity control",
    read: {
        "PlasticityQueueDepth": {
            path: "plasticity_queue_depth",
            description: "Get the current plasticity queue depth (number of pending plasticity operations)",
            response: {
                depth: i32, "Plasticity queue depth",
            },
        },
        "Status": {
            path: "status",
            description: "Get neuroplasticity status across all cortical areas including enabled state and queue depth",
            alloc_only: true,
            response: {
                global_plasticity_enabled: bool, "Whether plasticity is enabled globally",
                transforming_areas: Vec<String>, "IDs of cortical areas currently transforming",
                queue_depth: i32, "Plasticity queue depth",
            },
        },
        "Transforming": {
            path: "transforming",
            description: "Get list of cortical areas currently undergoing plasticity transformation",
            alloc_only: true,
            response: {
                areas: Vec<String>, "IDs of cortical areas currently transforming",
            },
        },
    },
    create: {
        "Configure": {
            path: "configure",
            description: "Configure neuroplasticity parameters including learning rates and plasticity rules",
            alloc_only: true,
            payload: {
                config: HashMap<String, serde_json::Value>, "Plasticity configuration to apply",
            },
            response: {
                message: String, "Result message",
            },
        },
        "EnableArea": {
            path: "enable/:area_id<String>",
            path_item_descriptions: {
                area_id: "Cortical area ID",
            },
            description: "Enable neuroplasticity for a specific cortical area",
            alloc_only: true,
            response: {
                message: String, "Result message",
            },
        },
        "DisableArea": {
            path: "disable/:area_id<String>",
            path_item_descriptions: {
                area_id: "Cortical area ID",
            },
            description: "Disable neuroplasticity for a specific cortical area",
            alloc_only: true,
            response: {
                message: String, "Result message",
            },
        },
    },
    edit: {
        "PlasticityQueueDepth": {
            path: "plasticity_queue_depth",
            description: "Set the plasticity queue depth to control how many operations can be pending",
            alloc_only: true,
            payload: {
                depth: i32, "New plasticity queue depth",
            },
            response: {
                message: String, "Result message",
            },
        },
    },
}

// NOTES:
// GET plasticity_queue_depth returned a bare i32; wrapped as `depth`.
// PUT plasticity_queue_depth took a bare i32 body; wrapped as `depth`.
// GET transforming returned a bare Vec<String>; wrapped as `areas`.
// Configure body was a whole untyped map (not under a key); wrapped as `config`, shape unknown.
// POST responses were bare HashMap<String, String> with one "message" key; mapped to `message`.
