use std::thread::JoinHandle;
use picoserve::Router;
use picoserve::routing::get;
use thiserror::Error;
use crate::error::FeagiInteropError;

pub struct RestServer {
   join_handle: JoinHandle<Result<(), FeagiRestServerError>>,
}

impl RestServer {

    pub async fn new_server<RestEndpoints>(socket_address: core::net::SocketAddr) -> Result<Self, FeagiInteropError> {
        // STD Variant

        static CONFIG: picoserve::Config = picoserve::Config::const_default().close_connection_after_response();

        let socket = std::net::TcpListener::bind(socket_address)
            .map_err(|_| FeagiInteropError::FailedToBindSocket)?;

        let mut router = picoserve::Router::new()
            .with_state();

        #[cfg(feature = "transport_rest_swagger")]
        {
            router.route("/", picoserve::routing::get(async || ))
        }

        // TODO add routes for Rest Endpoints (macro)

        let handle = std::thread::spawn(loop_rest_server(socket, router));

        Ok(Self { join_handle: handle })
    }
}

#[cfg(feature = "std")]
async fn loop_rest_server<RestEndpoints>(socket: std::net::TcpListener, router: Router<>) -> Result<(), FeagiRestServerError> {
    loop {
        match socket.accept() {
            Ok((stream, client_socket)) => {
                match picoserve::Server::serve(client_socket, router) {

                }
            }
            Err(_) => return Err(FeagiRestServerError::ServerParseFailure), // TODO ????
        }
    }
}

#[cfg(feature = "transport_rest_swagger")]
mod swagger {
    pub const HTML: &str = include_str!("../../resources_swagger/feagi-server.html");
    pub const CSS: &[u8] = include_bytes!("../../resources_swagger/swagger-ui.css");
    pub const JS: &[u8] = include_bytes!("../../resources_swagger/swagger-ui-bundle.js");
    pub const OPEN_API_JSON: &[u8] = include_bytes!("../../resources_swagger/openapi-server/openapi-json");
    // TODO CDN link on our cloud for deployments where we cannot fit the above data in the library
}

#[derive(Error, Debug)]
pub enum FeagiRestServerError {
    #[error("Server failed to parse data!")]
    ServerParseFailure
}


