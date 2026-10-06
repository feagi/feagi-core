use core::net::SocketAddrV4;
use core::net::Ipv4Addr;

use feagi_io_basis::contracts::request_response::request_response_set::RequestResponseEndpointSenderSet;
use feagi_io_basis::feagi_server_endpoint_and_config::{FeagiServerEndpointConfig};

/// Config struct used for Rest Servers
pub struct RestFeagiServerEndpointConfig<Requesters: RequestResponseEndpointSenderSet> {
    address: SocketAddrV4,
    requesters: Requesters
}

impl<Requesters: RequestResponseEndpointSenderSet> RestFeagiServerEndpointConfig<Requesters> {
    /// The default port the rest server runs on
    pub const DEFAULT_REST_SERVER_PORT: u16 = 8081;

    /// The default endpoint of the server
    pub const DEFAULT_REST_SERVER_ENDPOINT: SocketAddrV4 =
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, Self::DEFAULT_REST_SERVER_PORT);

    /// Create a new endpoint config
    pub fn new(address: SocketAddrV4, requesters: Requesters) -> Self {
        Self {
            address,
            requesters
        }
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        self.address
    }

    pub fn unwrap_requester(self) -> Requesters {
        self.requesters
    }

}

impl<Requesters: RequestResponseEndpointSenderSet>
FeagiServerEndpointConfig for RestFeagiServerEndpointConfig<Requesters>
{
    type RequestEndpointSenderSet = Requesters;
    type DataExchangeSet = ();
}


