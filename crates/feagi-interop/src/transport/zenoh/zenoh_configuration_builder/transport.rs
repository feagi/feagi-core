use super::error::ZenohConfigBuildError;

/// Unicast, multicast, link, shared memory, and authentication settings.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub struct ZenohTransportConfig {
    pub unicast: ZenohUnicastTransport,
    pub multicast: ZenohMulticastTransport,
    pub link: ZenohLinkTransport,
    pub shared_memory: ZenohSharedMemoryTransport,
    pub auth: ZenohTransportAuth,
}

impl ZenohTransportConfig {
    /// Checks unicast, link, shared-memory, and authentication constraints together.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.unicast.validate()?;
        self.link.validate()?;
        self.shared_memory.validate()?;
        self.auth.validate()
    }
}

/// Unicast session limits and features.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohUnicastTransport {
    /// Timeout in milliseconds when opening a link. Default: 10000.
    pub open_timeout: u64,
    /// Timeout in milliseconds when accepting a link. Default: 10000.
    pub accept_timeout: u64,
    /// Maximum number of links waiting to finish the accept handshake. Default: 100.
    pub accept_pending: u64,
    /// Maximum number of transports alive for one Zenoh session. Default: 1000.
    pub max_sessions: u64,
    /// Maximum number of incoming links admitted per transport. Default: 1.
    pub max_links: u64,
    /// Request the LowLatency transport.
    ///
    /// The peer still negotiates the transport that is actually used. LowLatency skips QoS
    /// prioritization and fragmentation, so a message must be smaller than
    /// [`ZenohLinkTx::batch_size`]. `qos.enabled` has to be false when this is true.
    pub lowlatency: bool,
    /// QoS on unicast sessions. Default: enabled.
    pub qos: ZenohEnabled,
    /// Compression negotiated while the session opens. Default: disabled.
    /// Active only when both peers support it.
    pub compression: ZenohEnabled,
}

impl Default for ZenohUnicastTransport {
    fn default() -> Self {
        Self {
            open_timeout: 10_000,
            accept_timeout: 10_000,
            accept_pending: 100,
            max_sessions: 1000,
            max_links: 1,
            lowlatency: false,
            qos: ZenohEnabled::enabled(),
            compression: ZenohEnabled::disabled(),
        }
    }
}

impl ZenohUnicastTransport {
    /// Rejects LowLatency while QoS is enabled. Zenoh requires QoS to be off for that transport.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        if self.lowlatency && self.qos.enabled {
            Err(ZenohConfigBuildError::LowLatencyWithQos)
        } else {
            Ok(())
        }
    }
}

/// Multicast session limits and features.
///
/// Multicast does not negotiate parameters when a node joins the group. Every node has to
/// use the same transport parameters. Link batch size is the usual one to align, because
/// Zenoh's platform default for that field differs by operating system.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohMulticastTransport {
    /// JOIN message interval in milliseconds. Default: 2500.
    pub join_interval: u64,
    /// Maximum number of multicast sessions. Default: 1000.
    pub max_sessions: u64,
    /// QoS on multicast. Default: disabled, for Zenoh-to-Zenoh-Pico compatibility.
    pub qos: ZenohEnabled,
    /// Compression on multicast. Default: disabled, for Zenoh-to-Zenoh-Pico compatibility.
    pub compression: ZenohEnabled,
}

impl Default for ZenohMulticastTransport {
    fn default() -> Self {
        Self {
            join_interval: 2500,
            max_sessions: 1000,
            qos: ZenohEnabled::disabled(),
            compression: ZenohEnabled::disabled(),
        }
    }
}

/// On/off flag for a transport feature such as QoS or compression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ZenohEnabled {
    /// Encoded as `{ enabled: <bool> }`.
    pub enabled: bool,
}

impl ZenohEnabled {
    /// `{ enabled: true }`.
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }

    /// `{ enabled: false }`.
    pub const fn disabled() -> Self {
        Self { enabled: false }
    }
}

/// Link protocol whitelist and transmit, receive, TLS, and TCP settings.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohLinkTransport {
    /// Protocols allowed when accepting and opening sessions.
    /// `None` allows every protocol Zenoh supports on this build.
    /// Encoded as `protocols`, for example `["tls", "quic"]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<ZenohLinkProtocol>>,
    pub tx: ZenohLinkTx,
    pub rx: ZenohLinkRx,
    pub tls: ZenohTlsLink,
    /// Present only when TCP socket buffers are set. Omitted from the document when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp: Option<ZenohTcpLink>,
}

impl Default for ZenohLinkTransport {
    fn default() -> Self {
        Self {
            protocols: None,
            tx: ZenohLinkTx::default(),
            rx: ZenohLinkRx::default(),
            tls: ZenohTlsLink::default(),
            tcp: None,
        }
    }
}

impl ZenohLinkTransport {
    /// Checks the protocol whitelist, batch size, queue sizes, and receive limits.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        if let Some(protocols) = &self.protocols {
            if protocols.is_empty() {
                return Err(ZenohConfigBuildError::EmptyProtocolList);
            }
        }
        self.tx.validate()?;
        self.rx.validate()?;
        if self.rx.max_message_size < u64::from(self.tx.batch_size) {
            return Err(ZenohConfigBuildError::MaxMessageSmallerThanBatch);
        }
        Ok(())
    }
}

/// Link protocol that a whitelist entry can name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ZenohLinkProtocol {
    /// `"tcp"`
    #[serde(rename = "tcp")]
    Tcp,
    /// `"udp"`
    #[serde(rename = "udp")]
    Udp,
    /// `"tls"`
    #[serde(rename = "tls")]
    Tls,
    /// `"quic"`
    #[serde(rename = "quic")]
    Quic,
    /// `"ws"`
    #[serde(rename = "ws")]
    WebSocket,
    /// `"unixsock-stream"`
    #[serde(rename = "unixsock-stream")]
    UnixSockStream,
    /// `"vsock"`
    #[serde(rename = "vsock")]
    Vsock,
}

/// Transmit parameters for a link.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohLinkTx {
    /// Sequence-number width announced to the peer. The session uses the smaller of the two sides.
    /// Default: 32-bit.
    pub sequence_number_resolution: ZenohSequenceNumberBits,
    /// Link lease announced to other nodes, in milliseconds. Default: 10000.
    pub lease: u64,
    /// Keep-alive messages sent during one lease when no data is flowing. Default: 4.
    pub keep_alive: u32,
    /// Batch size in bytes. Zenoh treats this as a 16-bit value, so the maximum is 65535.
    /// Default: 65535.
    pub batch_size: u16,
    pub queue: ZenohTxQueue,
}

impl Default for ZenohLinkTx {
    fn default() -> Self {
        Self {
            sequence_number_resolution: ZenohSequenceNumberBits::Bits32,
            lease: 10_000,
            keep_alive: 4,
            batch_size: u16::MAX,
            queue: ZenohTxQueue::default(),
        }
    }
}

impl ZenohLinkTx {
    /// Rejects a zero batch size and queue sizes outside 1 through 16.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        if self.batch_size == 0 {
            return Err(ZenohConfigBuildError::ZeroBatchSize);
        }
        self.queue.validate()
    }
}

/// Width of the sequence numbers announced on a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub enum ZenohSequenceNumberBits {
    /// `"8bit"`
    #[serde(rename = "8bit")]
    Bits8,
    /// `"16bit"`
    #[serde(rename = "16bit")]
    Bits16,
    /// `"32bit"`
    #[default]
    #[serde(rename = "32bit")]
    Bits32,
    /// `"64bit"`
    #[serde(rename = "64bit")]
    Bits64,
}

/// Transmit queue for one link.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub struct ZenohTxQueue {
    pub size: ZenohTxQueueSize,
    pub congestion_control: ZenohTxCongestionControl,
    pub batching: ZenohTxBatching,
    pub allocation: ZenohTxQueueAllocation,
}

impl ZenohTxQueue {
    /// Checks that every priority queue holds 1 to 16 batches.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.size.validate()
    }
}

/// Batches each priority queue can hold.
///
/// Each field is a count from 1 to 16. Memory for a queue is `count * batch_size`, or
/// `count * link_mtu` when the link MTU is smaller than the batch size. With QoS disabled,
/// only the `data` queue is allocated. Default for every priority: 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ZenohTxQueueSize {
    pub control: u8,
    pub real_time: u8,
    pub interactive_high: u8,
    pub interactive_low: u8,
    pub data_high: u8,
    pub data: u8,
    pub data_low: u8,
    pub background: u8,
}

impl Default for ZenohTxQueueSize {
    fn default() -> Self {
        Self {
            control: 2,
            real_time: 2,
            interactive_high: 2,
            interactive_low: 2,
            data_high: 2,
            data: 2,
            data_low: 2,
            background: 2,
        }
    }
}

impl ZenohTxQueueSize {
    /// Smallest batch count Zenoh accepts for one priority queue.
    pub const MIN_BATCHES: u8 = 1;
    /// Largest batch count Zenoh accepts for one priority queue.
    pub const MAX_BATCHES: u8 = 16;

    /// Rejects a priority whose batch count is outside 1 through 16.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        for (queue, value) in [
            ("control", self.control),
            ("real_time", self.real_time),
            ("interactive_high", self.interactive_high),
            ("interactive_low", self.interactive_low),
            ("data_high", self.data_high),
            ("data", self.data),
            ("data_low", self.data_low),
            ("background", self.background),
        ] {
            if !(Self::MIN_BATCHES..=Self::MAX_BATCHES).contains(&value) {
                return Err(ZenohConfigBuildError::QueueSize { queue, value });
            }
        }
        Ok(())
    }
}

/// Congestion behavior for the link transmit queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub struct ZenohTxCongestionControl {
    pub drop: ZenohTxDropCongestion,
    pub block: ZenohTxBlockCongestion,
}

/// Behavior for `CongestionControl::Drop` messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ZenohTxDropCongestion {
    /// Microseconds to wait for a free batch before dropping the message. Default: 1000.
    pub wait_before_drop: u64,
    /// Microsecond deadline limit for a multi-fragment message. Default: 50000.
    pub max_wait_before_drop_fragments: u64,
}

impl Default for ZenohTxDropCongestion {
    fn default() -> Self {
        Self {
            wait_before_drop: 1000,
            max_wait_before_drop_fragments: 50_000,
        }
    }
}

/// Behavior for `CongestionControl::Block` messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ZenohTxBlockCongestion {
    /// Microseconds to wait for a free batch before closing the session. Default: 5000000.
    pub wait_before_close: u64,
}

impl Default for ZenohTxBlockCongestion {
    fn default() -> Self {
        Self {
            wait_before_close: 5_000_000,
        }
    }
}

/// Groups small messages into one transmit batch under network back-pressure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ZenohTxBatching {
    /// Default: true.
    pub enabled: bool,
    /// Maximum time, in milliseconds, a message is held for batching. Default: 1.
    pub time_limit: u64,
}

impl Default for ZenohTxBatching {
    fn default() -> Self {
        Self {
            enabled: true,
            time_limit: 1,
        }
    }
}

/// When the transmit queue allocates its batches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub struct ZenohTxQueueAllocation {
    /// Encoded as `"init"` or `"lazy"`.
    pub mode: ZenohQueueAllocationMode,
}

/// When priority-queue batches are allocated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub enum ZenohQueueAllocationMode {
    /// Allocate every batch when the queue is created. Encoded as `"init"`.
    #[serde(rename = "init")]
    Init,
    /// Allocate a batch on demand, up to [`ZenohTxQueueSize`]. Encoded as `"lazy"`.
    #[default]
    #[serde(rename = "lazy")]
    Lazy,
}

/// Receive parameters for a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ZenohLinkRx {
    /// Receive buffer size in bytes for each link. Default: 65535.
    pub buffer_size: u64,
    /// Maximum defragmentation buffer at the receiver, in bytes.
    /// Larger fragmented messages are dropped. Default: 1GiB (1073741824).
    pub max_message_size: u64,
}

impl Default for ZenohLinkRx {
    fn default() -> Self {
        Self {
            buffer_size: 65_535,
            max_message_size: 1_073_741_824,
        }
    }
}

impl ZenohLinkRx {
    /// Rejects a zero receive buffer or a zero defragmentation limit.
    pub fn validate(self) -> Result<(), ZenohConfigBuildError> {
        if self.buffer_size == 0 {
            return Err(ZenohConfigBuildError::ZeroReceiveBuffer);
        }
        if self.max_message_size == 0 {
            return Err(ZenohConfigBuildError::ZeroMaxMessageSize);
        }
        Ok(())
    }
}

/// TLS certificate and key paths for a link.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohTlsLink {
    /// Path to the certificate authority. JSON `null` when unset.
    /// In router mode, unset selects the default WebPKI certificates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_ca_certificate: Option<String>,
    /// Path to the listening-side private key. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_private_key: Option<String>,
    /// Path to the listening-side certificate. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_certificate: Option<String>,
    /// Mutual TLS, so the listener also authenticates the client. Default: false.
    pub enable_mtls: bool,
    /// Path to the connecting-side private key. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_private_key: Option<String>,
    /// Path to the connecting-side certificate. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_certificate: Option<String>,
    /// Check that the remote certificate name matches the dialed host. Default: true.
    pub verify_name_on_connect: bool,
    /// Close TLS and QUIC links when the remote certificate chain expires.
    /// A listener only observes client expiry when mutual TLS is enabled. Default: false.
    pub close_link_on_expiration: bool,
    /// TCP receive buffer in bytes. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub so_rcvbuf: Option<u32>,
    /// TCP send buffer in bytes. Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub so_sndbuf: Option<u32>,
}

impl Default for ZenohTlsLink {
    fn default() -> Self {
        Self {
            root_ca_certificate: None,
            listen_private_key: None,
            listen_certificate: None,
            enable_mtls: false,
            connect_private_key: None,
            connect_certificate: None,
            verify_name_on_connect: true,
            close_link_on_expiration: false,
            so_rcvbuf: None,
            so_sndbuf: None,
        }
    }
}

/// Optional TCP socket buffer sizes, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub struct ZenohTcpLink {
    /// Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub so_rcvbuf: Option<u32>,
    /// Omitted when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub so_sndbuf: Option<u32>,
}

/// Shared-memory transport support.
///
/// These fields take effect when Zenoh is built with shared memory. The session still
/// probes the peer: both sides have to enable shared memory for payloads to stay off the network.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohSharedMemoryTransport {
    /// Announce shared-memory buffer support to peers. Default: true.
    pub enabled: bool,
    /// When the shared-memory subsystem is initialized. Default: lazy.
    pub mode: ZenohSharedMemoryInitMode,
    pub transport_optimization: ZenohSharedMemoryOptimization,
}

impl Default for ZenohSharedMemoryTransport {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ZenohSharedMemoryInitMode::Lazy,
            transport_optimization: ZenohSharedMemoryOptimization::default(),
        }
    }
}

impl ZenohSharedMemoryTransport {
    /// Checks that an enabled transport optimization has a non-zero pool.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.transport_optimization.validate()
    }
}

/// When the shared-memory subsystem is initialized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub enum ZenohSharedMemoryInitMode {
    /// Initialize when the session opens. Encoded as `"init"`.
    #[serde(rename = "init")]
    Init,
    /// Initialize on the first shared-memory buffer. Encoded as `"lazy"`.
    #[default]
    #[serde(rename = "lazy")]
    Lazy,
}

/// Copies large ordinary payloads into shared memory for links that can carry it.
///
/// Payloads the application already allocated in shared memory are sent that way whenever
/// the peer supports it, independent of `messages`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ZenohSharedMemoryOptimization {
    /// Default: true.
    pub enabled: bool,
    /// Shared-memory pool size in bytes. Default: 16MiB (16777216).
    pub pool_size: u64,
    /// Smallest payload, in bytes, that this optimization will move. Default: 3072.
    pub message_size_threshold: u64,
    /// Message categories copied into shared memory. Default: put, query, and reply.
    /// `delete` has no payload, so listing it has no effect.
    pub messages: Vec<ZenohTransportDataMessage>,
}

impl Default for ZenohSharedMemoryOptimization {
    fn default() -> Self {
        Self {
            enabled: true,
            pool_size: 16_777_216,
            message_size_threshold: 3072,
            messages: vec![
                ZenohTransportDataMessage::Put,
                ZenohTransportDataMessage::Query,
                ZenohTransportDataMessage::Reply,
            ],
        }
    }
}

impl ZenohSharedMemoryOptimization {
    /// Rejects an enabled optimization whose pool size is zero.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        if self.enabled && self.pool_size == 0 {
            Err(ZenohConfigBuildError::EmptySharedMemoryPool)
        } else {
            Ok(())
        }
    }
}

/// Message category eligible for shared-memory transport optimization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ZenohTransportDataMessage {
    /// `"put"`
    #[serde(rename = "put")]
    Put,
    /// `"delete"`
    #[serde(rename = "delete")]
    Delete,
    /// `"query"`
    #[serde(rename = "query")]
    Query,
    /// `"reply"`
    #[serde(rename = "reply")]
    Reply,
}

/// Authentication material for the transport.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub struct ZenohTransportAuth {
    pub usrpwd: ZenohUserPasswordAuth,
    pub pubkey: ZenohPublicKeyAuth,
}

impl ZenohTransportAuth {
    /// Checks that a password is paired with a username.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.usrpwd.validate()
    }
}

/// Username and password authentication. A password requires a username.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub struct ZenohUserPasswordAuth {
    /// Username. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Password. JSON `null` when unset. Requires [`Self::user`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Path to a file of username/password entries. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dictionary_file: Option<String>,
}

impl ZenohUserPasswordAuth {
    /// Rejects a non-empty password whose username is missing or empty.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        let user_missing = match &self.user {
            Some(user) => user.is_empty(),
            None => true,
        };
        let password_set = match &self.password {
            Some(password) => !password.is_empty(),
            None => false,
        };
        if password_set && user_missing {
            Err(ZenohConfigBuildError::PasswordWithoutUser)
        } else {
            Ok(())
        }
    }
}

/// Public-key authentication material.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub struct ZenohPublicKeyAuth {
    /// PEM public key. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_pem: Option<String>,
    /// PEM private key. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_pem: Option<String>,
    /// Path to the public key file. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_file: Option<String>,
    /// Path to the private key file. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_file: Option<String>,
    /// Key size. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_size: Option<u32>,
    /// Path to the known-keys file. JSON `null` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_keys_file: Option<String>,
}
