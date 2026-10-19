use super::system_requests::*;
use feagi_basis::feagi_basis_proc_common::define_template_request_categories;

define_template_request_categories! {
    exported_macro_name: feagi_design_server_request_categories,
    template: {
        group_name: BrainServer,
        categories: {
            system: template_design_server_system_requests,
        }
    }
}
