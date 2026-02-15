use crate::FeagiNetworkError;
use crate::traits_and_enums::shared::{TransportProtocolEndpoint, TransportProtocolImplementation};
use crate::traits_and_enums::server::{FeagiServer};

/// A server that receives pushed data from clients.
///
/// Implements the pull side of the push-pull messaging pattern. Clients push
/// data to this server, which collects it for processing.
pub trait FeagiServerPuller: FeagiServer {
    /// Consumes and returns the retrieved data from clients.
    ///
    /// # Lifetime
    ///
    /// The returned slice is valid only for the duration of this call. The data
    /// must be copied or fully processed before calling any other method on this
    /// server, as the internal buffer may be reused.
    ///
    /// # State Requirements
    ///
    /// Only call when `poll()` returns `ActiveHasData`.
    ///
    /// # Errors
    ///
    /// Returns an error if no data is available or if retrieval fails.
    fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError>;

    /// Creates a boxed properties object for this puller.
    ///
    /// This allows decoupling the configuration/properties from the active
    /// puller instance, enabling creation of new pullers with the same
    /// configuration.
    fn as_boxed_puller_properties(&self) -> Box<dyn FeagiServerPullerProperties>;
}

pub trait FeagiServerPullerProperties: Send + Sync {
    /// Creates a new boxed puller from these properties.
    fn as_boxed_server_puller(&self) -> Box<dyn FeagiServerPuller>;

    /// Gets the local bind point
    fn get_bind_point(&self) -> TransportProtocolEndpoint;

    /// Gets the bind point that is given to agents (the remote bind point)
    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint;

    // What protocols do both endpoints use?
    fn get_protocol(&self) -> TransportProtocolImplementation;

}