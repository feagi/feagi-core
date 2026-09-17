
// types are just 


macro_rules! define_request_response {
    (
        server_request_response,
        permissionss: [Read, Write],

        requests_responses {
            
            burst_engine: [
                {
                    path_extension: "simulation_timestep",
                    description: "Gets the frequency timestep (in seconds) between bursts"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "simulation_timestep",
                    description: "Sets the frequency timestep (in seconds) between bursts"
                    permissions: [],
                    rest_type: post,
                },

                {
                    path_extension: "stats",
                    description: "Gets the current stats of the burst engine"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "status",
                    description: "Gets the current status of the burst engine"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "control",
                    description: "Controls the burst engine"
                    permissions: [],
                    rest_type: post,
                },

                {
                    path_extension: "burst_counter",
                    description: "Gets the number of bursts elapsed since genome load"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "start",
                    description: "Starts the burst engine"
                    permissions: [],
                    rest_type: post,
                },

                {
                    path_extension: "stop",
                    description: "Stops the burst engine"
                    permissions: [],
                    rest_type: post,
                },

                {
                    path_extension: "hold",
                    description: "Holds the burst engine"
                    permissions: [],
                    rest_type: post,
                },

                {
                    path_extension: "resume",
                    description: "Resumes the burst engine"
                    permissions: [],
                    rest_type: post,
                },

                {
                    path_extension: "config",
                    description: "Gets the burst engine config"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "config",
                    description: "Updates the burst engine config"
                    permissions: [],
                    rest_type: put,
                },
            ],

            connectome: [
                {
                    path_extension: "cortical_areas/list/detailed",
                    description: "TODO"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "properties/dimensions",
                    description: "TODO"
                    permissions: [],
                    rest_type: get,
                },

                {
                    path_extension: "properties/mappings",
                    description: "TODO"
                    permissions: [],
                    rest_type: get,
                },
                
                
            ]
            
            
        }



    ) => {};
}