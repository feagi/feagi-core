pub const HTML: &str = include_str!("../../../resources_swagger/feagi-server.html");
pub const CSS: &[u8] = include_bytes!("../../../resources_swagger/swagger-ui.css");
pub const JS: &[u8] = include_bytes!("../../../resources_swagger/swagger-ui-bundle.js");

// `from_template_make_open_api_json_generator_function` emits `generate_open_api_str`,
// whose result is the document to serve here.
// pub const OPEN_API_JSON: &str = generate_open_api_str();
// TODO CDN link on our cloud for deployments where we cannot fit the above data in the library
