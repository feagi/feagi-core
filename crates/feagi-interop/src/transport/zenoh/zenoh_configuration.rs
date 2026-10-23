use core::net::SocketAddr;


// TODO scouting options, for now scouting should be disabled
// TODO other transports, for now all other transports should be disabled, only ws for now
pub struct ZenohConfigurationBuilder {
    building_config: ZenohConfiguration
}

impl ZenohConfigurationBuilder {

    pub fn new() -> Self {
        Self {building_config: Default::default()}
    }

    #[cfg(feature = "transport_ws")]
    pub fn set_ws_server_bind(&mut self, socket: SocketAddr) {
        self.building_config.ws_server_bind = Some(socket)
    }

    #[cfg(feature = "transport_ws")]
    pub fn set_ws_client_target(&mut self, socket: SocketAddr) {
        self.building_config.ws_client_target = Some(socket)
    }

    pub fn build(self) -> Result<ZenohConfiguration, ZenohConfigurationError> {
        // TODO proper error checking
        Ok(self.building_config)
    }
}

impl Default for ZenohConfigurationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A unified Zenoh configuration struct. Starts with nothing enabled. Use the functions to
/// enable systems one by one. Note that rust crate level features can disable certain
/// functionality of zenoh and thus would be disabled here.
#[derive(Default, Clone, Debug)]
pub struct ZenohConfiguration {
    /// TODO We are exposing the internal data structs for the future case where if we serialize
    /// a config that is calling for features that arent enabled, we can return a descript
    /// error later
    ws_server_bind: Option<SocketAddr>,
    ws_client_target: Option<SocketAddr>


}




/// As there is a std and no-std zenoh crate that use different config structs, this trait unifies
/// the usage of our Zenoh Configuration to become the type of struct the zenoh crate enabled
/// expects
pub(crate) trait ZenohConfigurationBackend {
    /// The type of config /  struct the version of the zenoh crate enabled expects
    type BackendType;

    fn build_zenoh_config(config: ZenohConfiguration) -> Result<Self::BackendType, ZenohConfigurationError>;
}


#[derive(Debug, thiserror::Error)]
pub enum ZenohConfigurationError {
    #[error("Failed To Build Configuration!")]
    FailedToBuildConfiguration
    // TODO flesh this out
}