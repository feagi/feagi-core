//! Plasticity: temporary/debug plasticity executor controls.
//!
//! Converted from the old `plasticity` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

define_request_message_category! {
    category_name: "Plasticity",
    base_path: "v1/plasticity",
    category_description: "Temporary/debug plasticity executor controls",
    create: {
        "RegisterMemoryArea": {
            path: "register_memory_area",
            description: "Manually register a memory area with the plasticity executor (temporary debug endpoint)",
            alloc_only: true,
            response: {
                result: HashMap<String, serde_json::Value>,
            },
        },
    },
}

// NOTES:
// - Old endpoint was an unimplemented stub (always errored); request body was an untyped map never read, so payload is omitted (likely a memory cortical area ID).
// - Response was an untyped map; wrapped as `result`.
