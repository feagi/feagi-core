
// TODO later, we need to proxy requests through this manager instead of directly to endpoints
// TODO we need to prox through here to handle permissions and security (rbac?)

use core::net::SocketAddrV4;
use feagi_io_basis::contracts::request_response::request_response_set::RequestResponseEndpointSenderSet;
use feagi_io_basis::feagi_server_endpoint_and_config::{FeagiFailStartRestServerEtc, FeagiRestServerStartError, FeagiServerEndpoint, FeagiServerEndpointLauncher};


pub struct FeagiServers {
    #[cfg(feature = "rest-server")]
    rest_server: Option<FeagiServerEndpoint>
}

impl FeagiServers {

    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rest-server")]
            rest_server: None,
        }
    }

    #[cfg(feature = "rest-server")]
    /// Starts a web server at a given address. Returns an error if this fails or if a web server
    /// is already running
    pub fn start_web_server<Requesters: RequestResponseEndpointSenderSet>(&mut self, requests: Requesters, socket: SocketAddrV4) -> Result<(), FeagiRestServerStartError> {
        if self.rest_server.is_some() {
            return Err(FeagiRestServerStartError::Etc(FeagiFailStartRestServerEtc::new("Server already running!")))
        }
        use feagi_io_rest::feagi_rest_basis::rest_feagi_server_endpoint_config::RestFeagiServerEndpointConfig;

        let config = RestFeagiServerEndpointConfig::new(socket, requests);

        let running_server_endpoint = {
            #[cfg(feature = "std")]
            {
                // STD web server is Ohkami
                use feagi_io_rest::feagi_rest_server_std::OhkamiServerEndpointLauncher;
                OhkamiServerEndpointLauncher::launch_server_endpoint(config)?
            }
            #[cfg(not(feature = "std"))]
            {
                todo!()
            }
        };
        self.rest_server = Some(running_server_endpoint);
        Ok(())
    }

    /// Stops all servers, one at a time
    pub async fn stop_all_servers(&mut self) {
        // TODO is it possible to do this in unison? Await together?
        #[cfg(feature = "rest-server")]
        {
            let maybe_server = self.rest_server.take();
            if let Some(server) = maybe_server {
                server.stop_server().await
            }
        }

    }


}

