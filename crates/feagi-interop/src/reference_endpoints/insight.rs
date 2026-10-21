//! Insight: inspecting and setting neuron potentials, and analytics data for debugging.
//!
//! Converted from the old `insight` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

define_request_message_category! {
    category_name: "Insight",
    base_path: "v1/insight",
    category_description: "Inspecting and setting neuron potentials, and analytics data for debugging",
    read: {
        "Analytics": {
            path: "analytics",
            description: "Get analytics data for neural activity and performance insights",
            alloc_only: true,
            response: {
                analytics: HashMap<String, serde_json::Value>,
            },
        },
        "Data": {
            path: "data",
            description: "Get detailed insight data for debugging and analysis",
            alloc_only: true,
            response: {
                data: HashMap<String, serde_json::Value>,
            },
        },
    },
    create: {
        "NeuronsMembranePotentialStatus": {
            path: "neurons/membrane_potential_status",
            description: "Get membrane potential status for multiple neurons",
            alloc_only: true,
            response: {
                potentials: HashMap<String, f32>, "Membrane potential per neuron",
            },
        },
        "NeuronSynapticPotentialStatus": {
            path: "neuron/synaptic_potential_status",
            description: "Get synaptic potential status for a specific neuron",
            alloc_only: true,
            response: {
                status: serde_json::Value,
            },
        },
        "NeuronsMembranePotentialSet": {
            path: "neurons/membrane_potential_set",
            description: "Set membrane potential for multiple neurons",
            alloc_only: true,
            response: {
                result: HashMap<String, String>,
            },
        },
        "NeuronSynapticPotentialSet": {
            path: "neuron/synaptic_potential_set",
            description: "Set synaptic potential for a specific neuron",
            alloc_only: true,
            response: {
                result: HashMap<String, String>,
            },
        },
    },
}

// NOTES:
// - All four POST endpoints were unimplemented stubs taking an untyped map body that was never read; payloads omitted (fields unknown).
// - All responses were bare maps / Values; wrapped (`analytics`, `data`, `potentials`, `status`, `result`).
// - Analytics / Data returned empty maps; shape unknown.
// - The "status" endpoints are POST in the old API, so they live in `create`.
