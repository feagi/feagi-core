
/*

format:
{
    $GENERATED_MACRO_NAME,
    $ROOT_PATH_NAME,
    request_response: {
        $REGION_NAME: {
            read: {
                ($PATH_EXTENSION): {
                    description: "$DESCRIPTION",
                    request: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                    response: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                },
            },
            create: {
                ($PATH_EXTENSION): {
                    description: "$DESCRIPTION",
                    request: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                    response: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                },
            },
            edit: {
                ($PATH_EXTENSION): {
                    description: "$DESCRIPTION",
                    request: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                    response: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                },
            },
            delete: {
                ($PATH_EXTENSION): {
                    description: "$DESCRIPTION",
                    request: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                    response: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                },
            },
            patch: {
                ($PATH_EXTENSION): {
                    description: "$DESCRIPTION",
                    request: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                    response: [
                        "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                        ...
                    ],
                },
            },
        }
    }
}
 */


macro_rules! define_requests_to_server {
    (
        requests_to_server,
        v2
        request_response {
            burst_engine: {
                read: {
                    (simulation_timestep): {
                        description: "Gets the frequency timestep (in seconds) between bursts",
                        request: [],
                        response: []
                    },
                    (stats): {
                        description: "Gets the current stats of the burst engine",
                        request: [],
                        response: []
                    },
                    (status): {
                        description: "Gets the current status of the burst engine",
                        request: [],
                        response: []
                    },
                    (burst_counter): {
                        description: "Gets the number of bursts elapsed since genome load",
                        request: [],
                        response: []
                    },
                    (config): {
                        description: "Gets the burst engine config",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (simulation_timestep): {
                        description: "Sets the frequency timestep (in seconds) between bursts",
                        request: [],
                        response: []
                    },
                    (control): {
                        description: "Controls the burst engine",
                        request: [],
                        response: []
                    },
                    (start): {
                        description: "Starts the burst engine",
                        request: [],
                        response: []
                    },
                    (stop): {
                        description: "Stops the burst engine",
                        request: [],
                        response: []
                    },
                    (hold): {
                        description: "Holds the burst engine",
                        request: [],
                        response: []
                    },
                    (resume): {
                        description: "Resumes the burst engine",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (config): {
                        description: "Updates the burst engine config",
                        request: [],
                        response: []
                    },
                },
                delete: {},
                patch: {},
            },
            connectome: {
                read: {
                    (cortical_areas/list/detailed): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (properties/dimensions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (properties/mappings): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (snapshot): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron_count): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (synapse_count): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (paths): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cumulative_stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (area_details): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (structure): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (validation): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (topology): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (connectivity_matrix): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_areas/list/summary): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_areas/list/transforming): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area/{cortical_id}/neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({cortical_area_id}/synapses): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({cortical_area_id}/synapses/incoming): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_info/{cortical_area}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (stats/cortical_area/cumulative/{cortical_area}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron/{neuron_id}/properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron_properties_at): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (area_neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (plasticity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (path): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (download): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (download-cortical_area-area/{cortical_area}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (batch_neuron_operations): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (batch_synapse_operations): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (rebuild): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (clear): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (optimize): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neurons/batch): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (synapses/batch): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload-cortical_area-area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            cortical_area: {
                read: {
                    (ipu): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (opu): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_id_list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_name_list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_id_name_mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_map_detailed): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_locations_2d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area/geometry): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_visibility): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (visualization): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (ipu/list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (opu/list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (voxel_neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (memory): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (ipu/types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (opu/types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_index_list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_idx_mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_restrictions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({cortical_id}/memory_usage): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({cortical_id}/neuron_count): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (cortical_name_location): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (multi/cortical_area_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (custom_cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (clone): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (batch_operations): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (resize): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reposition): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (voxel_neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_type_options): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_restrictions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_restrictions_between_areas): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (multi/cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (coord_2d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (suppress_cortical_visibility): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (coordinates_3d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (coord_3d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (multi/cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (bulk_delete): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            cortical_mapping: {
                read: {
                    (mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (afferents): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (efferents): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (batch_update): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (mapping_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            evolution: {
                read: {
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            genome: {
                read: {
                    (file_name): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (circuits): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (name): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (timestamp): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (download): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (metadata): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (diff): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (amalgamation): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (amalgamation_history): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_template): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (defaults/files): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (download_region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (genome_number): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (amalgamation_destination): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (append): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload/barebones): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload/essential): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (save): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (load): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (validate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (transform): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (clone): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reset): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (merge): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (export_format): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (amalgamation_by_filename): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (amalgamation_by_payload): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (amalgamation_by_upload): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (append-file): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload/file): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload/file/edit): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload/string): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {
                    (amalgamation_cancellation): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            input: {
                read: {
                    (vision): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (sources): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (sensor_snapshot/last): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (vision): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (configure): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            insight: {
                read: {
                    (analytics): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (data): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {},
                edit: {},
                delete: {},
                patch: {},
            },
            monitoring: {
                read: {
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (metrics): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (data): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (performance): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_activity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {},
                edit: {},
                delete: {},
                patch: {},
            },
            morphology: {
                read: {
                    (morphology_list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (morphology_types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (list/types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (morphologies): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (info/{morphology_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (morphology): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (morphology_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (morphology_usage): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (create): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (morphology): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (rename): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (update): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (morphology): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (delete/{morphology_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            network: {
                read: {
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (connection_info): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            neuroplasticity: {
                read: {
                    (plasticity_queue_depth): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (transforming): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (configure): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (enable/{area_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (disable/{area_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (plasticity_queue_depth): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {},
                patch: {},
            },
            outputs: {
                read: {
                    (targets): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (motor_snapshot/last): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (configure): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            physiology: {
                read: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {},
                edit: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {},
                patch: {},
            },
            plasticity: {
                read: {},
                create: {
                    (register_memory_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            region: {
                read: {
                    (regions_members): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (regions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (region_titles): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (region/{region_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (clone): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (relocate_members): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (change_region_parent): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (change_cortical_area_region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (region_and_members): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            simulation: {
                read: {
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (upload/string): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reset): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },
            system: {
                read: {
                    (health_check): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_visualization_skip_rate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_visualization_suppression_threshold): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (version): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (versions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (configuration): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (user_preferences): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (processes): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (unique_logs): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (log_tail): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (beacon/subscribers): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (global_activity_visualization): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (db/influxdb/test): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (enable_visualization_fq_sampler): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (disable_visualization_fq_sampler): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (logs): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (beacon/subscribe): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (circuit_library_path): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (register): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (cortical_area_visualization_skip_rate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_visualization_suppression_threshold): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (user_preferences): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (global_activity_visualization): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (beacon/unsubscribe): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            training: {
                read: {
                    (shock/options): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (shock/status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (brain_fitness): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fitness_criteria): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fitness_stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (training_report): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (shock): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reward/intensity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (punishment/intensity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (gameover): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reward): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (punishment): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (shock/activate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fitness_criteria): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (fitness_criteria): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fitness_stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (fitness_stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reset_fitness_stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {},
            },
            visualization: {
                read: {
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (register_client): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (unregister_client): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (heartbeat): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {},
                delete: {},
                patch: {},
            },

        }

    ) => {};
}