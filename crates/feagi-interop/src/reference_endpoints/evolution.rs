//! Evolution: evolutionary algorithm status and configuration.
//!
//! Converted from the old `evolution` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

define_request_message_category! {
    category_name: "Evolution",
    base_path: "v1/evolution",
    category_description: "Evolutionary algorithm status and configuration",
    read: {
        "Status": {
            path: "status",
            description: "Get evolution system status including generation, population size, and activity state",
            response: {
                active: bool, "Whether evolution is running",
                generation: u64, "Current generation",
                population_size: u64, "Size of the population",
            },
        },
    },
    create: {
        "Config": {
            path: "config",
            description: "Configure evolution parameters including mutation rates and selection criteria",
            alloc_only: true,
            payload: {
                config: HashMap<String, serde_json::Value>, "Evolution parameters",
            },
            response: {
                message: String,
            },
        },
    },
}

// NOTES:
// - Config `config` is an untyped map in the old DTO (EvolutionConfigRequest); its keys are unknown.
// - Status population_size was usize in the old DTO; u64 here.
// - Both endpoints were stubs (status hardcoded, config ignored).
