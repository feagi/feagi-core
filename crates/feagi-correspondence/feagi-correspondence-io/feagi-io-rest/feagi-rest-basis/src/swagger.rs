
/// The actual data for including the data in the binary itself
pub const SWAGGER_HTML: &str = include_str!("../swagger_site/feagi-server.html");
pub const SWAGGER_CSS: &[u8] = include_bytes!("../swagger_site/swagger-ui.css");
pub const SWAGGER_JS: &[u8] = include_bytes!("../swagger_site/swagger-ui-bundle.js");

// TODO CDN link on our cloud for deployments where we cannot fit the above data in the library



