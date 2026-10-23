//! Lowers a [`ZenohConfiguration`] into the config of the std zenoh crate

use super::zenoh_configuration_builder::configuration_builder::ZenohSTDConfigurationBuilder;
use super::zenoh_configuration_builder::connect::ZenohEndpointEntry;
use super::zenoh_configuration_builder::error::ZenohConfigBuildError;
use crate::transport::zenoh::zenoh_configuration::{ZenohConfiguration, ZenohConfigurationBackend};

/// Lowers a [`ZenohConfiguration`] into a `zenoh::Config`
///
/// The node mode follows from what is enabled. A websocket server alone is a router, as other
/// nodes connect through it. A websocket client alone is a client. Both together is a peer that
/// listens and connects. Everything not enabled is off, including the listeners and scouting
/// zenoh enables by default
pub struct ZenohStdConfigurationBackend;

impl ZenohConfigurationBackend for ZenohStdConfigurationBackend {
    type BackendType = zenoh::Config;
    type BackendError = ZenohConfigBuildError;

    fn build_zenoh_config(config: ZenohConfiguration) -> Result<zenoh::Config, ZenohConfigBuildError> {
        let listen = config.ws_server_bind().map(|socket| vec![ws_endpoint(socket)]).unwrap_or_default();
        let connect = config.ws_client_target().map(|socket| vec![ws_endpoint(socket)]).unwrap_or_default();

        let builder = ZenohSTDConfigurationBuilder::default();
        let mut node = match (listen.is_empty(), connect.is_empty()) {
            (false, true) => builder.into_router(),
            (true, false) => builder.into_client(),
            _ => builder.into_peer(),
        };
        // Replaces the default listeners of the mode, so only what was enabled is bound
        node.set_listen_endpoints(listen)?;
        node.set_connect_endpoints(connect)?;

        let mut zenoh_config = node.to_zenoh_config()?;
        // TODO move scouting into the json builder. Zenoh scouts by default, which would connect
        // nodes over other transports than the enabled ones
        for key in ["scouting/multicast/enabled", "scouting/gossip/enabled"] {
            zenoh_config
                .insert_json5(key, "false")
                .map_err(|error| ZenohConfigBuildError::Json5(error.to_string()))?;
        }
        Ok(zenoh_config)
    }
}

fn ws_endpoint(socket: core::net::SocketAddr) -> ZenohEndpointEntry {
    ZenohEndpointEntry::Locator(format!("ws/{socket}"))
}
