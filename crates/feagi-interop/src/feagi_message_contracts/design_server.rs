use feagi_basis::feagi_basis_proc_common::define_template_request_categories;
use feagi_interop_proc::from_template_make_open_api_json_generator_function;

define_template_request_categories! {
    exported_macro_name: design_server_request_categories,
    template: {
        group_name: DesignServer,
        categories: [
            system
        ]
    }
}

design_server_request_categories!(from_template_make_open_api_json_generator_function);