
// TODO later, we need to proxy requests through this manager instead of directly to endpoints
// TODO we need to prox through here to handle permissions and security (rbac?)

use core::net::SocketAddrV4;
use feagi_io_basis::feagi_server_endpoint_and_config::{FeagiServerEndpoint, FeagiServerEndpointLauncher};


pub struct IOManager {
    #[cfg(feature = "rest-server")]
    rest_server: Option<FeagiServerEndpoint>
}

impl IOManager {

    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rest-server")]
            rest_server: None,
        }
    }

    #[cfg(feature = "rest-server")]
    pub fn start_web_server(&mut self, requests: (), socket: SocketAddrV4) {

        use feagi_io_rest::feagi_rest_basis::rest_feagi_server_endpoint_config::RestFeagiServerEndpointConfig;


        let config = RestFeagiServerEndpointConfig::new(socket, requests);

        let running_server_endpoint = {
            #[cfg(feature = "std")]
            {
                use feagi_io_rest::feagi_rest_server_std::OhkamiServerEndpointLauncher;
                OhkamiServerEndpointLauncher::launch_server_endpoint(config)?;
            }
            #[cfg(not(feature = "std"))]
            {
                todo!()
            }
        };
        self.rest_server = Some(running_server_endpoint)
    }

    pub async fn stop_all_servers(&mut self) {

        #[cfg(feature = "rest-server")]
        {
            let maybe_server = self.rest_server.take();
            if let Some(server) = maybe_server {
                server.stop_server().await
            }
        }

    }


}

