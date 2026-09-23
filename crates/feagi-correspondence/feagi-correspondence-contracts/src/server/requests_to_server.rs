
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
            agent: {
                read: {
                    (capabilities): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (capabilities/all): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fq_sampler_status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (info/{agent_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (liveness): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (properties/{agent_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (shared_mem): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({agent_id}/device_registrations): {
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
                    (heartbeat): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (manual_stimulation): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (register): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({agent_id}/device_registrations): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    (deregister): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            system: {
                read: {
                    (beacon/subscribers): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (configuration): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_types): {
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
                    (db/influxdb/test): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (global_activity_visualization): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (health_check): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (log_tail): {
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
                    (user_preferences): {
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
                },
                create: {
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
                    (disable_visualization_fq_sampler): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (enable_visualization_fq_sampler): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_reset): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (logs): {
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
                    (global_activity_visualization): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (user_preferences): {
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
                patch: {
                },
            },
            cortical_area: {
                read: {
                    (cortical_area/geometry): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_id_list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_index_list): {
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
                    (cortical_idx_mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_locations_2d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_map_detailed): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_visibility): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (ipu): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (ipu/list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (ipu/types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_restrictions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (memory): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (opu): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (opu/list): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (opu/types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (visualization): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (voxel_neurons): {
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
                    (batch_operations): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (clone): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_name_location): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_type_options): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (custom_cortical_area): {
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
                    (multi/cortical_area_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reposition): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (resize): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (voxel_neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (coord_2d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (coord_3d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (coordinates_3d): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                    (reset): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (suppress_cortical_visibility): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                    (bulk_delete): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                },
                patch: {
                },
            },
            morphology: {
                read: {
                    (info/{morphology_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (list): {
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
                },
                create: {
                    (create): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                    (delete/{morphology_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (morphology): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            region: {
                read: {
                    (region/{region_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (region_titles): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (regions): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (regions_members): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (clone): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (change_cortical_area_region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (change_region_parent): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                patch: {
                },
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
                    (batch_update): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (efferents): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (mapping): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (mapping_properties): {
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
                patch: {
                },
            },
            connectome: {
                read: {
                    (area_details): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (area_neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (connectivity_matrix): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area/list/types): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_area/{cortical_id}/neurons): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cortical_areas/list/detailed): {
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
                    (cortical_info/{cortical_area}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (cumulative_stats): {
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
                    (fire_queue/{cortical_area}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (memory_neuron): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron/{neuron_id}/properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron_count): {
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
                    (path): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (paths): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (plasticity): {
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
                    (stats/cortical_area/cumulative/{cortical_area}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (structure): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (synapse_count): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (topology): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (validation): {
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
                    (clear): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neurons/batch): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (optimize): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (rebuild): {
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
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            burst_engine: {
                read: {
                    (burst_counter): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl/neuron): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_sampler/area/{area_id}/sample_rate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_sampler/config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_ledger/area/{area_id}/history): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_ledger/area/{area_id}/window_size): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_ledger/areas_window_config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_ledger/default_window_size): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_queue): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_queue/neuron): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (frequency_history): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (frequency_status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (membrane_potentials): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (simulation_timestep): {
                        description: "Gets the frequency timestep (in seconds) between bursts",
                        request: [],
                        response: []
                    },
                    (stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (control): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_reset): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_sampler/area/{area_id}/sample_rate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fcl_sampler/config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (force_connectome_integration): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (hold): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (measure_frequency): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (resume): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (simulation_timestep): {
                        description: "Sets the frequency timestep (in seconds) between bursts",
                        request: [],
                        response: []
                    },
                    (start): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (stop): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                    (config): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_ledger/area/{area_id}/window_size): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (fire_ledger/default_window_size): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (membrane_potentials): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                },
                patch: {
                },
            },
            genome: {
                read: {
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
                    (circuits): {
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
                    (diff): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (download): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (download_region): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (file_name): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (genome_number): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (metadata): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (name): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (properties): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (timestamp): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
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
                    (amalgamation_destination): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (append-file): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (clone): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (export_format): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (load): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (merge): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reset): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (save): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (transform): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload): {
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
                    (validate): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    (amalgamation_cancellation): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            feagi: {
                read: {
                },
                create: {
                    (genome/append): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
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
                    (disable/{area_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (enable/{area_id}): {
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
                delete: {
                },
                patch: {
                },
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
                create: {
                    (neuron/synaptic_potential_set): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neuron/synaptic_potential_status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neurons/membrane_potential_set): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (neurons/membrane_potential_status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            input: {
                read: {
                    (sensor_snapshot/last): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (sources): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (vision): {
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
                    (vision): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            output: {
                read: {
                    (motor_snapshot/last): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (targets): {
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
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            physiology: {
                read: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                },
                edit: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                delete: {
                },
                patch: {
                },
            },
            simulation: {
                read: {
                    (stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                    (reset): {
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
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            training: {
                read: {
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
                    (stats): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (training_report): {
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
                    (fitness_criteria): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (gameover): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (punishment): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (punishment/intensity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reward): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (reward/intensity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (shock): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (shock/activate): {
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
                patch: {
                },
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
                    (heartbeat): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            monitoring: {
                read: {
                    (cortical_activity): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (data): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (metrics): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (performance): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (status): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
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
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            network: {
                read: {
                    (connection_info): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
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
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            : {
                read: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            api-docs: {
                read: {
                    (openapi.json): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },
            snapshot: {
                read: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({snapshot_id}/artifact/{fmt}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (compare): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (create): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (restore): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (upload): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    ({snapshot_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            snapshots: {
                read: {
                    ({snapshot_id}/artifact/{fmt}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                    (connectome): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    (connectome/{snapshot_id}/restore): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                    ({snapshot_id}/restore): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    ({snapshot_id}): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            swagger-ui: {
                read: {
                    (): {
                        description: "TODO",
                        request: [],
                        response: []
                    },
                },
                create: {
                },
                edit: {
                },
                delete: {
                },
                patch: {
                },
            },

        }

    ) => {};
}
