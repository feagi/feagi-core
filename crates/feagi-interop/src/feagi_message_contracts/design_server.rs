use feagi_basis::feagi_basis_proc_common::define_template_request_categories;
use feagi_interop_proc::from_template_make_open_api_json_generator_function;

#[cfg(feature = "open_api_endpoint")]
use crate::rest::server::open_api::{CategoryApi, OpenApiDocument};

define_template_request_categories! {
    exported_macro_name: design_server_request_categories,
    template: {
        group_name: DesignServer,
        categories: [
            system
        ]
    }
}

fn a(d: OpenApiDocument) {

}

#[cfg(feature = "open_api_endpoint")]
design_server_request_categories!(from_template_make_open_api_json_generator_function);