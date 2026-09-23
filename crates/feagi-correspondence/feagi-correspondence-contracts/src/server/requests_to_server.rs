
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
                    }
                },
                create: {
                    (simulation_timestep): {
                        description: "Sets the frequency timestep (in seconds) between bursts",
                        request: [],
                        response: []
                    }
                }
            },
            connectome: {
                read: {
                    (cortical_areas/list/detailed): [
                        description: "TODO",
                        request: [],
                        response: []
                    ]
                }
            }

        }
        
    ) => {};
}