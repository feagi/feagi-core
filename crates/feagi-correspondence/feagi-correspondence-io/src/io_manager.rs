
// TODO later, we need to proxy requests through this manager instead of directly to endpoints
// TODO we need to prox through here to handle permissions and security (rbac?)

use core::net::SocketAddrV4;
use feagi_io_basis::feagi_server_endpoint_and_config::FeagiServerEndpoint;


pub struct IOManager {
    rest_server: Option<FeagiServerEndpoint>
}

impl IOManager {

    pub fn new() -> Self {
        Self {
            rest_server: None
        }
    }

    pub fn start_web_server(request_endpoints: (), socket: SocketAddrV4) -> Self {

        let mut rest_config = OhkamiFeagiServerEndpointConfig::new(request_endpoints);
        rest_config.set_specific_socket(socket);

        let rest_server = FeagiServerEndpoint::launch_new_server(rest_config);

        Self {
            rest_server: Some(rest_server)
        }

    }


    pub fn stop_all_servers(&mut self) {
        if let Some(rest) = &mut self.rest_server {
            rest.stop_server()
        }
    }


}

