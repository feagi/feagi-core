
/*

format:
{
    exported_macro_name: $GENERATED_MACRO_NAME,
    template: {
        $ROOT_PATH_NAME,
        requests_responses: {
            $REGION_NAME: {
                read: {
                    "$PATH_EXTENSION": {
                        description: "$DESCRIPTION",
                        request_parameters: [
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
                    "$PATH_EXTENSION": {
                        description: "$DESCRIPTION",
                        request_parameters: [
                            "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                            ...
                        ],
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
                    "$PATH_EXTENSION": {
                        description: "$DESCRIPTION",
                        request_parameters: [
                            "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                            ...
                        ],
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
                    "$PATH_EXTENSION": {
                        description: "$DESCRIPTION",
                        request_parameters: [
                            "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                            ...
                        ],
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
                    "$PATH_EXTENSION": {
                        description: "$DESCRIPTION",
                        request_parameters: [
                            "$PARAMETER_NAME": $VALUE_TYPE_SUPPORTING_BASE64_URL_ENCODING,
                            ...
                        ],
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
}
 */


macro_rules! define_requests_to_server {
    (
        requests_to_server,
        "v2",
        request_response {
            "agent": {
                read: {
                    "capabilities": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "capabilities/all": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fq_sampler_status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "info/{agent_id}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "liveness": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "properties": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "properties/{agent_id}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "shared_mem": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "{agent_id}/device_registrations": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "configure": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "heartbeat": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "manual_stimulation": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "register": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "{agent_id}/device_registrations": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    "deregister": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "system": {
                read: {
                    "beacon/subscribers": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "configuration": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area_types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area_visualization_skip_rate": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area_visualization_suppression_threshold": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "db/influxdb/test": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fcl_status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "global_activity_visualization": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "health_check": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "log_tail": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "processes": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "unique_logs": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "user_preferences": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "version": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "versions": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "beacon/subscribe": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "circuit_library_path": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "disable_visualization_fq_sampler": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "enable_visualization_fq_sampler": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fcl_reset": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "logs": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "register": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "cortical_area_visualization_skip_rate": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_area_visualization_suppression_threshold": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "global_activity_visualization": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "user_preferences": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                    "beacon/unsubscribe": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "cortical_area": {
                read: {
                    "cortical_area/geometry": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area_id_list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area_index_list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area_name_list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_id_name_mapping": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_idx_mapping": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_locations_2d": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_map_detailed": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_visibility": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "ipu": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "ipu/list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "ipu/types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "mapping_restrictions": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "memory": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "opu": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "opu/list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "opu/types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "visualization": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "voxel_neurons": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "{cortical_id}/memory_usage": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "{cortical_id}/neuron_count": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "batch_operations": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "clone": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_area": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_area_properties": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_name_location": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_type_options": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "custom_cortical_area": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "mapping_restrictions": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "mapping_restrictions_between_areas": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "multi/cortical_area_properties": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reposition": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "resize": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "voxel_neurons": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "coord_2d": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "coord_3d": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "coordinates_3d": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_area": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "multi/cortical_area": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reset": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "suppress_cortical_visibility": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                    "bulk_delete": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "cortical_area": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "multi/cortical_area": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "morphology": {
                read: {
                    "info/{morphology_id}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "list/types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "morphologies": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "morphology_list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "morphology_types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "create": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "morphology": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "morphology_properties": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "morphology_usage": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "morphology": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "rename": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "update": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                    "delete/{morphology_id}": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "morphology": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "region": {
                read: {
                    "region/{region_id}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "region_titles": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "regions": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "regions_members": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "clone": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "region": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "change_cortical_area_region": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "change_region_parent": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "region": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "relocate_members": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                    "region": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "region_and_members": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "cortical_mapping": {
                read: {
                    "mapping": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "mapping_list": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "afferents": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "batch_update": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "efferents": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "mapping": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "mapping_properties": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "mapping": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "mapping_properties": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                    "mapping": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "connectome": {
                read: {
                    "area_details": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "area_neurons": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "connectivity_matrix": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area/list/types": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_area/{cortical_id}/neurons": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_areas/list/detailed": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_areas/list/summary": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_areas/list/transforming": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_info/{cortical_area}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cumulative_stats": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "download": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "download-cortical_area-area/{cortical_area}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_queue/{cortical_area}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "memory_neuron": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "neuron/{neuron_id}/properties": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "neuron_count": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "neuron_properties": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "neuron_properties_at": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "path": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "paths": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "plasticity": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "properties/dimensions": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "properties/mappings": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "snapshot": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "stats": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "stats/cortical_area/cumulative/{cortical_area}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "structure": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "synapse_count": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "topology": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "validation": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "{cortical_area_id}/synapses": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "{cortical_area_id}/synapses/incoming": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "batch_neuron_operations": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "batch_synapse_operations": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "clear": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "neurons/batch": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "optimize": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "rebuild": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "synapses/batch": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload-cortical_area-area": {
                        description: "TODO",
                        request_parameters: [],
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
            "burst_engine": {
                read: {
                    "burst_counter": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "config": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fcl": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fcl/neuron": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fcl_sampler/area/{area_id}/sample_rate": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fcl_sampler/config": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fcl_status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_ledger/area/{area_id}/history": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_ledger/area/{area_id}/window_size": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_ledger/areas_window_config": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_ledger/default_window_size": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_queue": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fire_queue/neuron": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "frequency_history": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "frequency_status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "membrane_potentials": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "simulation_timestep": {
                        description: "Gets the frequency timestep (in seconds) between bursts",
                        request_parameters: [],
                        response: []
                    },
                    "stats": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "control": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fcl_reset": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fcl_sampler/area/{area_id}/sample_rate": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fcl_sampler/config": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "force_connectome_integration": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "hold": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "measure_frequency": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "resume": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "simulation_timestep": {
                        description: "Sets the frequency timestep (in seconds) between bursts",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "start": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "stop": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "config": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fire_ledger/area/{area_id}/window_size": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fire_ledger/default_window_size": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "membrane_potentials": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                },
                patch: {
                },
            },
            "genome": {
                read: {
                    "amalgamation": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "amalgamation_history": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "circuits": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "cortical_template": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "defaults/files": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "diff": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "download": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "download_region": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "file_name": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "genome_number": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "metadata": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "name": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "properties": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "timestamp": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "amalgamation_by_filename": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "amalgamation_by_payload": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "amalgamation_by_upload": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "amalgamation_destination": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "append-file": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "clone": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "export_format": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "load": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "merge": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reset": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "save": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "transform": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload/barebones": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload/essential": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload/file": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload/file/edit": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload/string": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "validate": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    "amalgamation_cancellation": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "feagi": {
                read: {
                },
                create: {
                    "genome/append": {
                        description: "TODO",
                        request_parameters: [],
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
            "neuroplasticity": {
                read: {
                    "plasticity_queue_depth": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "transforming": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "configure": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "disable/{area_id}": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "enable/{area_id}": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "plasticity_queue_depth": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                },
                patch: {
                },
            },
            "insight": {
                read: {
                    "analytics": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "data": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "neuron/synaptic_potential_set": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "neuron/synaptic_potential_status": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "neurons/membrane_potential_set": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "neurons/membrane_potential_status": {
                        description: "TODO",
                        request_parameters: [],
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
            "input": {
                read: {
                    "sensor_snapshot/last": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "sources": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "vision": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "configure": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "vision": {
                        description: "TODO",
                        request_parameters: [],
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
            "output": {
                read: {
                    "motor_snapshot/last": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "targets": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "configure": {
                        description: "TODO",
                        request_parameters: [],
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
            "physiology": {
                read: {
                    "": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                },
                edit: {
                    "": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                },
                patch: {
                },
            },
            "simulation": {
                read: {
                    "stats": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "config": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reset": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload/string": {
                        description: "TODO",
                        request_parameters: [],
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
            "training": {
                read: {
                    "brain_fitness": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fitness_criteria": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "fitness_stats": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "shock/options": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "shock/status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "stats": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "training_report": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "config": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fitness_criteria": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "gameover": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "punishment": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "punishment/intensity": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reward": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reward/intensity": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "shock": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "shock/activate": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                    "fitness_criteria": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "fitness_stats": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                delete: {
                    "fitness_stats": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "reset_fitness_stats": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            "visualization": {
                read: {
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "heartbeat": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "register_client": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "unregister_client": {
                        description: "TODO",
                        request_parameters: [],
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
            "monitoring": {
                read: {
                    "cortical_activity": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "data": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "metrics": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "performance": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "status": {
                        description: "TODO",
                        request_parameters: [],
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
            "evolution": {
                read: {
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "config": {
                        description: "TODO",
                        request_parameters: [],
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
            "network": {
                read: {
                    "connection_info": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "status": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "config": {
                        description: "TODO",
                        request_parameters: [],
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
            "api-docs": {
                read: {
                    "openapi.json": {
                        description: "TODO",
                        request_parameters: [],
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
            "snapshot": {
                read: {
                    "": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                    "{snapshot_id}/artifact/{fmt}": {
                        description: "TODO",
                        request_parameters: [],
                        response: []
                    },
                },
                create: {
                    "compare": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "create": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "restore": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                    "upload": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                edit: {
                },
                delete: {
                    "{snapshot_id}": {
                        description: "TODO",
                        request_parameters: [],
                        request: [],
                        response: []
                    },
                },
                patch: {
                },
            },
            swagger-ui: {
                read: {
                    "": {
                        description: "TODO",
                        request_parameters: [],
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
