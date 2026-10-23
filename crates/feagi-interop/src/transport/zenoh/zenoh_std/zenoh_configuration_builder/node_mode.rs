/// Role of this node in the Zenoh network.
#[derive(Clone, Copy, Debug, Hash)]
pub enum ZenohNodeMode {
    #[cfg(feature = "std")]
    /// Connects directly to other nodes. Encoded as `"peer"`.
    Peer,
    #[cfg(feature = "std")]
    /// Routes between nodes. Encoded as `"router"`.
    Router,
    /// Dials one router or peer. Encoded as `"client"`.
    Client,
}

impl ZenohNodeMode {
    /// Encoded as the JSON string `mode`.
    pub fn as_json_str(self) -> &'static str {
        match self {
            #[cfg(feature = "std")]
            Self::Peer => "peer",
            #[cfg(feature = "std")]
            Self::Router => "router",
            Self::Client => "client",
        }
    }
}

impl Default for ZenohNodeMode {
    #[cfg(feature = "std")]
    fn default() -> Self {
        ZenohNodeMode::Peer
    }

    #[cfg(not(feature = "std"))]
    fn default() -> Self {
        ZenohNodeMode::Client
    }
}
