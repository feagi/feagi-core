use super::configuration_builder::ZenohConfigurationBuilder;
use super::connect::{ZenohConnectConfig, ZenohEndpointEntry, ZenohEndpointRetry, ZenohListenConfig, ZenohTimeoutMs};
use super::error::ZenohConfigBuildError;
use super::node_mode::ZenohNodeMode;
use super::transport::ZenohTransportConfig;

/// Connect and listen settings for one already chosen node mode.
///
/// Produced by [`super::configuration_builder::ZenohConfigurationBuilder::into_client`], and on `std` by
/// `into_peer` and `into_router`. [`Self::build_json5`] writes the Zenoh JSON5 document.
#[derive(Clone, Debug)]
pub struct ZenohNodeBuilder {
    configuration: ZenohConfigurationBuilder,
    mode: ZenohNodeMode,
    connect: ZenohConnectConfig,
    listen: ZenohListenConfig,
}

impl ZenohNodeBuilder {
    pub(super) fn new(mode: ZenohNodeMode, configuration: ZenohConfigurationBuilder) -> Self {
        Self {
            configuration,
            connect: ZenohConnectConfig::for_mode(mode),
            listen: ZenohListenConfig::for_mode(mode),
            mode,
        }
    }

    /// Settings chosen before the node mode.
    pub fn configuration(&self) -> &ZenohConfigurationBuilder {
        &self.configuration
    }

    /// Router, peer, or client. Fixed when this builder is created.
    pub fn mode(&self) -> ZenohNodeMode {
        self.mode
    }

    /// Endpoints this node dials.
    pub fn connect(&self) -> &ZenohConnectConfig {
        &self.connect
    }

    /// Endpoints this node binds.
    pub fn listen(&self) -> &ZenohListenConfig {
        &self.listen
    }

    /// Replaces connect settings for this mode.
    pub fn set_connect(&mut self, connect: ZenohConnectConfig) -> Result<&mut Self, ZenohConfigBuildError> {
        connect.validate()?;
        self.connect = connect;
        Ok(self)
    }

    /// Sets how long connect waits for its endpoints.
    pub fn set_connect_timeout(&mut self, timeout_ms: ZenohTimeoutMs) -> Result<&mut Self, ZenohConfigBuildError> {
        timeout_ms.validate()?;
        self.connect.timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Sets the endpoints connect dials.
    pub fn set_connect_endpoints(&mut self, endpoints: Vec<ZenohEndpointEntry>) -> Result<&mut Self, ZenohConfigBuildError> {
        validate_endpoint_list(&endpoints)?;
        self.connect.endpoints = endpoints;
        Ok(self)
    }

    /// Sets whether connect exits the process when its timeout elapses.
    pub fn set_connect_exit_on_failure(&mut self, exit_on_failure: bool) -> &mut Self {
        self.connect.exit_on_failure = exit_on_failure;
        self
    }

    /// Sets the backoff connect uses between dial attempts.
    pub fn set_connect_retry(&mut self, retry: ZenohEndpointRetry) -> Result<&mut Self, ZenohConfigBuildError> {
        retry.validate()?;
        self.connect.retry = retry;
        Ok(self)
    }

    /// Replaces listen settings for this mode.
    pub fn set_listen(&mut self, listen: ZenohListenConfig) -> Result<&mut Self, ZenohConfigBuildError> {
        listen.validate()?;
        self.listen = listen;
        Ok(self)
    }

    /// Sets how long listen waits for its endpoints to bind.
    pub fn set_listen_timeout(&mut self, timeout_ms: ZenohTimeoutMs) -> Result<&mut Self, ZenohConfigBuildError> {
        timeout_ms.validate()?;
        self.listen.timeout_ms = timeout_ms;
        Ok(self)
    }

    /// Sets the endpoints listen binds.
    pub fn set_listen_endpoints(&mut self, endpoints: Vec<ZenohEndpointEntry>) -> Result<&mut Self, ZenohConfigBuildError> {
        validate_endpoint_list(&endpoints)?;
        self.listen.endpoints = endpoints;
        Ok(self)
    }

    /// Sets whether listen exits the process when its timeout elapses.
    pub fn set_listen_exit_on_failure(&mut self, exit_on_failure: bool) -> &mut Self {
        self.listen.exit_on_failure = exit_on_failure;
        self
    }

    /// Sets the backoff listen uses between bind attempts.
    pub fn set_listen_retry(&mut self, retry: ZenohEndpointRetry) -> Result<&mut Self, ZenohConfigBuildError> {
        retry.validate()?;
        self.listen.retry = retry;
        Ok(self)
    }

    /// Checks connect, listen, namespace, and transport together.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.connect.validate()?;
        self.listen.validate()?;
        if let Some(namespace) = self.configuration.namespace() {
            namespace.validate()?;
        }
        self.configuration.transport().validate()
    }

    /// Encodes this node as a JSON5 document. JSON text is valid JSON5.
    ///
    /// Available when the `serde_json` feature is enabled, which the `std` feature turns on.
    #[cfg(feature = "serde_json")]
    pub fn build_json5(&self) -> Result<String, ZenohConfigBuildError> {
        self.validate()?;
        let document = ZenohJson5Document {
            id: self.configuration.id_mode().json_id(),
            mode: self.mode.as_json_str(),
            connect: &self.connect,
            listen: &self.listen,
            queries_default_timeout: self.configuration.queries_default_timeout().milliseconds,
            namespace: self.configuration.namespace().map(|namespace| namespace.prefix.as_str()),
            transport: self.configuration.transport(),
        };
        serde_json::to_string(&document).map_err(|error| ZenohConfigBuildError::Json5(error.to_string()))
    }

    /// Actually export the config for Zenoh
    #[cfg(all(feature = "serde_json", feature = "std"))] // no-std zenoh is another crate
    pub fn to_zenoh_config(&self) -> Result<zenoh::Config, ZenohConfigBuildError> {
        let json = self.build_json5()?;
        zenoh::Config::from_json5(json.as_str()).map_err(|_| ZenohConfigBuildError::Json5(json))
    }
}

fn validate_endpoint_list(endpoints: &[ZenohEndpointEntry]) -> Result<(), ZenohConfigBuildError> {
    for endpoint in endpoints {
        endpoint.validate()?;
    }
    Ok(())
}

/// Top-level Zenoh JSON5 object for one node.
#[cfg(feature = "serde_json")]
#[derive(serde::Serialize)]
struct ZenohJson5Document<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    mode: &'a str,
    connect: &'a ZenohConnectConfig,
    listen: &'a ZenohListenConfig,
    queries_default_timeout: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<&'a str>,
    transport: &'a ZenohTransportConfig,
}
