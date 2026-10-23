use super::error::ZenohConfigBuildError;
use super::id_generation_mode::ZenohIDGenerationMode;
use super::namespace::ZenohNamespace;
use super::node::ZenohNodeBuilder;
use super::node_mode::ZenohNodeMode;
use super::queries_default_timeout::ZenohQueriesDefaultTimeout;
use super::transport::{
    ZenohLinkTransport, ZenohMulticastTransport, ZenohSharedMemoryTransport, ZenohTransportAuth, ZenohTransportConfig, ZenohUnicastTransport,
};

/// Settings that are the same whichever node mode is chosen later.
///
/// Call `into_client' (or `into_peer` / `into_router` on `std`) to pick the mode
/// and continue with connect and listen settings.
#[derive(Clone, Debug, Default)]
pub struct ZenohSTDConfigurationBuilder {
    id_mode: ZenohIDGenerationMode,
    namespace: Option<ZenohNamespace>,
    queries_default_timeout: ZenohQueriesDefaultTimeout,
    transport: ZenohTransportConfig,
}

impl ZenohSTDConfigurationBuilder {
    /// How the Zenoh ID is chosen.
    pub fn id_mode(&self) -> ZenohIDGenerationMode {
        self.id_mode
    }

    /// Session key-expression prefix.
    pub fn namespace(&self) -> Option<&ZenohNamespace> {
        self.namespace.as_ref()
    }

    /// Query timeout.
    pub fn queries_default_timeout(&self) -> ZenohQueriesDefaultTimeout {
        self.queries_default_timeout
    }

    /// Link, shared-memory, and authentication settings.
    pub fn transport(&self) -> &ZenohTransportConfig {
        &self.transport
    }

    /// Selects how the Zenoh ID is chosen.
    pub fn set_id_mode(&mut self, mode: ZenohIDGenerationMode) -> &mut Self {
        self.id_mode = mode;
        self
    }

    /// Sets the session key-expression prefix.
    pub fn set_namespace(&mut self, namespace: ZenohNamespace) -> Result<&mut Self, ZenohConfigBuildError> {
        namespace.validate()?;
        self.namespace = Some(namespace);
        Ok(self)
    }

    /// Removes the session key-expression prefix.
    pub fn clear_namespace(&mut self) -> &mut Self {
        self.namespace = None;
        self
    }

    /// Sets the query timeout, in milliseconds.
    pub fn set_queries_default_timeout(&mut self, milliseconds: u64) -> &mut Self {
        self.queries_default_timeout = ZenohQueriesDefaultTimeout { milliseconds };
        self
    }

    /// Replaces transport settings after checking incompatible combinations.
    pub fn set_transport(&mut self, transport: ZenohTransportConfig) -> Result<&mut Self, ZenohConfigBuildError> {
        transport.validate()?;
        self.transport = transport;
        Ok(self)
    }

    /// Replaces unicast transport settings.
    pub fn set_unicast(&mut self, unicast: ZenohUnicastTransport) -> Result<&mut Self, ZenohConfigBuildError> {
        unicast.validate()?;
        self.transport.unicast = unicast;
        Ok(self)
    }

    /// Replaces multicast transport settings.
    pub fn set_multicast(&mut self, multicast: ZenohMulticastTransport) -> &mut Self {
        self.transport.multicast = multicast;
        self
    }

    /// Replaces link transport settings.
    pub fn set_link(&mut self, link: ZenohLinkTransport) -> Result<&mut Self, ZenohConfigBuildError> {
        link.validate()?;
        self.transport.link = link;
        Ok(self)
    }

    /// Replaces shared-memory transport settings.
    pub fn set_shared_memory(&mut self, shared_memory: ZenohSharedMemoryTransport) -> Result<&mut Self, ZenohConfigBuildError> {
        shared_memory.validate()?;
        self.transport.shared_memory = shared_memory;
        Ok(self)
    }

    /// Replaces transport authentication settings.
    pub fn set_auth(&mut self, auth: ZenohTransportAuth) -> Result<&mut Self, ZenohConfigBuildError> {
        auth.validate()?;
        self.transport.auth = auth;
        Ok(self)
    }

    /// Continues as a client.
    pub fn into_client(self) -> ZenohNodeBuilder {
        self.into_mode(ZenohNodeMode::Client)
    }

    /// Continues as a peer.
    #[cfg(feature = "std")]
    pub fn into_peer(self) -> ZenohNodeBuilder {
        self.into_mode(ZenohNodeMode::Peer)
    }

    /// Continues as a router.
    #[cfg(feature = "std")]
    pub fn into_router(self) -> ZenohNodeBuilder {
        self.into_mode(ZenohNodeMode::Router)
    }

    fn into_mode(self, mode: ZenohNodeMode) -> ZenohNodeBuilder {
        ZenohNodeBuilder::new(mode, self)
    }
}
