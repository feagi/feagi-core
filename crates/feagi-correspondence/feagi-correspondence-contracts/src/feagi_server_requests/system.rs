#[cfg(feature = "feagi-server-requests")]
use feagi_basis::feagi_macros::request_responses::{generate_from_template_request_response_structs_and_enums, template_request_category};


#[cfg(feature = "feagi-server-requests")]
template_request_category! {
    exported_macro_name: feagi_server_requests_system,
    template: {
        category_name: "System",
        base_path: "system",
        category_description: "FEAGI Server System Level Statuses",
        async_functions_module: system,
        read: {
            "health_check": {
                title: "HealthCheck",
                description: "Checks current status of the FEAGI server",
                path_parameters: [],
                request: [],
                response: [
                    "agent_data_hash": i32, "Changes if connected agents changes",
                    "brain_geometry_hash": i32, "Changes if genome geometry changes",
                    "brain_regions_hash": i32, "Changes if brain regions changes",
                    "classifiers_hash": i32, "Changes if classifiers change",
                    "morphologies_hash": i32, "Changes if morphologies change",
                    "brain_readiness": bool, "Is the brain ready",
                    "burst_engine": bool, "Is the burst engine ready",
                    "genome_availability": bool, "Is the genome loaded and ready",
                    "genome_loading": bool, "Is the genome loading",
                    "genome_validity": bool, "Is the genome valid",
                    "memory_neuron_count": i32, "How many memory neurons are there currently",
                    "neuron_count": i32, "How many neurons are there currently in total",
                    "regular_neuron_count": i32, "How many non-memory neurons are there currently",
                    "synapse_count": i32, "How many synapses are there currently",
                ],
            }
        },
        create: {},
        edit: {},
        delete: {},
        patch: {}
    }
}

#[cfg(feature = "feagi-server-requests")]
feagi_server_requests_system!(generate_from_template_request_response_structs_and_enums);

