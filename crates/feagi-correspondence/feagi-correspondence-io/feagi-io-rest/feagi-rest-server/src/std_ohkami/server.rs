use ohkami::prelude::*;
use core::net::SocketAddrV4;
use core::net::Ipv4Addr;
use std::thread::JoinHandle;
use ohkami::claw::content::Html;
use ohkami::openapi;
use thingbuf::mpsc::Receiver;
use feagi_correspondence_contracts::feagi_server_requests::system::{FeagiRequestReceiveSystemError, SystemResponsesEnum};
use feagi_io_basis::config::{FeagiServerEndpointCommand, FeagiServerEndpointConfig};

const DEFAULT_WEB_SERVER_ADDRESS: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8081);

pub struct OhkamiFeagiServerEndpointConfig {
    address: SocketAddrV4,
    requests: Self::RequestModules
}

impl OhkamiFeagiServerEndpointConfig {
    /// Creates a new instance of this config with the default URL / Port settings
    pub fn new(requests: Self::RequestModules) -> Self {
        Self {
            address: DEFAULT_WEB_SERVER_ADDRESS,
            requests
        }
    }
    
    /// Set a specific address and port over the default
    pub fn set_specific_socket(&mut self, socket: SocketAddrV4) {
        self.address = socket;
    }
}

impl FeagiServerEndpointConfig for OhkamiFeagiServerEndpointConfig {
    type RequestModules = thingbuf::mpsc::Sender<Result<SystemResponsesEnum, FeagiRequestReceiveSystemError>>;
    type DataExchangeModules = (); // No data exchange supported by rest

    fn launch_server_thread(self, command_channel: Receiver<FeagiServerEndpointCommand>) -> JoinHandle<()> {

        fn ohkami_server(address: SocketAddrV4, requests: Self::RequestModules, command_channel: Receiver<FeagiServerEndpointCommand>) -> () {
            let ohkami_server = Ohkami::new((
                "/swagger-ui".GET(redirect_swagger),
                "/swagger-ui/feagi-server.html".GET(swagger_html),
                "/swagger-ui/swagger-ui.css".GET(swagger_css),
                "/swagger-ui/swagger-ui-bundle.js".GET(swagger_js),
                "/api-docs/openapi.json".GET(openapi_json),
                
                // TODO macro adds other paths here, compiles the data down to a request, sends it to the corresponding Self::RequestModules endpoint
            ));

            smol::block_on(async {
                ohkami_server.howl(address).await;
            })
        }
        
        let address = self.address.to_owned();
        let requests = self.requests.to_owned();

        let handle = std::thread::spawn(ohkami_server(address, requests, command_channel));
        
        handle
    }
    
    
}


//region OpenAPI generation and Swagger

const SWAGGER_HTML: &str = include_str!("../../swagger_site/feagi-server.html");
const SWAGGER_CSS: &[u8] = include_bytes!("../../swagger_site/swagger-ui.css");
const SWAGGER_JS: &[u8] = include_bytes!("../../swagger_site/swagger-ui-bundle.js");

fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
    openapi::OpenAPI {
        title: "FEAGI REST Server",
        version: env!("CARGO_PKG_VERSION"),
        servers: &[],
    }
}


fn openapi_json_bytes() -> &'static [u8] {
    static SPEC: OnceLock<Vec<u8>> = OnceLock::new();
    SPEC.get_or_init(|| create_ohkami_server().__openapi_document_bytes__(rest_openapi_metadata()))
}



async fn swagger_html() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

async fn swagger_css() -> Response {
    Response::OK().with_payload("text/css; charset=UTF-8", SWAGGER_CSS)
}

async fn swagger_js() -> Response {
    Response::OK().with_payload("application/javascript; charset=UTF-8", SWAGGER_JS)
}

async fn openapi_json() -> Response {
    Response::OK().with_payload("application/json", openapi_json_bytes())
}

async fn redirect_swagger() -> Response {
    Response::Found().with_headers(|h| h.location("/swagger-ui/feagi-server.html"))
}

//endregion







/*
use ohkami::prelude::*;
use ohkami::openapi;
use feagi_basis::feagi_macros::request_responses::{
    generate_from_template_ohkami_rest_server,
    template_request_category,
};

// Ohkami structs, async handlers, and `create_system_ohkami()`.
feagi_server_requests_system!(generate_from_template_ohkami_rest_server);

/// Hand-written async functions backing each `System` endpoint.
mod system_handlers {
    use super::*;

    pub async fn read_health_check(_request: ReadHealthCheckRequest) -> ReadHealthCheckResponse {
        // Placeholder: real health evaluation goes here.
        ReadHealthCheckResponse { status: String::from("ok") }
    }

    pub async fn read_health_check2(_request: ReadHealthCheck2Request) -> ReadHealthCheck2Response {
        ReadHealthCheck2Response { is_healthy: true }
    }
}

/// OpenAPI document metadata used by [`generate_openapi_document`].
fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
    openapi::OpenAPI {
        title: "FEAGI REST Server",
        version: env!("CARGO_PKG_VERSION"),
        servers: &[],
    }
}

/// Build the FEAGI REST Ohkami app. The caller owns serving it (this crate is a lib).
pub fn create_ohkami_server() -> Ohkami {
    create_system_ohkami()
}

/// Write this server's OpenAPI document to `file_path` (JSON).
pub fn generate_openapi_document(file_path: impl AsRef<std::path::Path>) {
    create_ohkami_server().generate_to(file_path, rest_openapi_metadata());
}


 */