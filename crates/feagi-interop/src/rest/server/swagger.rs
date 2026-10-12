

pub const HTML: &str = include_str!("../../resources_swagger/feagi-server.html");
pub const CSS: &[u8] = include_bytes!("../../resources_swagger/swagger-ui.css");
pub const JS: &[u8] = include_bytes!("../../resources_swagger/swagger-ui-bundle.js");

// TODO generate this statically, link directly (build via proc macro generated function from parsed endpoints)
//pub const OPEN_API_JSON: &[u8] = include_bytes!("../../resources_swagger/openapi-server/openapi-json");
// TODO CDN link on our cloud for deployments where we cannot fit the above data in the library

