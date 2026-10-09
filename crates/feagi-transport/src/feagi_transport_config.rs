use core::net::SocketAddr;


pub struct FeagiTransportConfig {
    #[cfg(feature = "transport_rest")]
    web_server_bind: Option<SocketAddr>,
    #[cfg(feature = "transport_rest_swagger")]
    expose_swagger_docs: bool,
}

impl FeagiTransportConfig {

    #[cfg(feature = "transport_rest")]
    pub fn set_rest_endpoint(&mut self, bind: Option<SocketAddr>)
    {
        #[cfg(feature = "transport_rest_swagger")]
        if bind.is_none() {
            self.expose_swagger_docs = false
        }
        self.web_server_bind = bind
    }

    #[cfg(feature = "transport_rest_swagger")]
    pub fn set_swagger_enabled(&mut self, enable_swagger: bool) {
        self.expose_swagger_docs = enable_swagger
    }

    // TODO enable websocket

    // TODO Enable other stuff


    /*


    pub fn generate_zenoh_config(&self) -> zenoh::Config {

        let mut config = zenoh::Config::default();

        #[cfg(feature = "transport_rest")]
        if let Some(web_server_bind) = self.web_server_bind {
            let port = web_server_bind.port();
            let config_payload = serde_json::json!({
                "plugins": {
                    "rest": {
                        "http_port": port
                    }
                }
            });
            config.insert_json5("plugins/rest/http_port", &port.to_string()).unwrap();
        };

        config

    }

     */
}