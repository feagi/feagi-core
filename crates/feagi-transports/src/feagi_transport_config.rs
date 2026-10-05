use core::net::SocketAddr;


pub struct FeagiTransportConfig {
    #[cfg(feature = "transport_rest")]
    web_server_bind: Option<SocketAddr>,
    #[cfg(feature = "transport_rest_swagger")]
    expose_swagger_docs: bool,
}

impl FeagiTransportConfig {
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
}