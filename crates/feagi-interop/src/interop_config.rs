use core::net::SocketAddr;

// TODO Server VS Client binding

// TODO Scouting?

// TODO Enable other stuff

/// Configuration Definition for Interop
#[derive(Debug, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct InteropConfig {
    #[cfg(feature = "transport_rest")]
    rest_server_bind: Option<SocketAddr>,
    #[cfg(feature = "transport_ws")]
    websocket_bind: Option<SocketAddr>,
    #[cfg(feature = "transport_quic")]
    quic_bind: Option<SocketAddr>,
}

impl InteropConfig {
    pub fn new_no_config() -> Self {
        Self {
            #[cfg(feature = "transport_rest")]
            rest_server_bind: None,
            #[cfg(feature = "transport_ws")]
            websocket_bind: None,
            #[cfg(feature = "transport_quic")]
            quic_bind: None,
        }
    }

    #[cfg(feature = "transport_rest")]
    pub fn set_rest_server(&mut self, addr: Option<SocketAddr>) -> &mut Self {
        self.rest_server_bind = addr;
        self
    }

    #[cfg(feature = "transport_ws")]
    pub fn set_websocket_bind(&mut self, addr: Option<SocketAddr>) -> &mut Self {
        self.websocket_bind = addr;
        self
    }

    #[cfg(feature = "transport_quic")]
    pub fn set_quic_bind(&mut self, addr: Option<SocketAddr>) -> &mut Self {
        self.quic_bind = addr;
        self
    }
}
