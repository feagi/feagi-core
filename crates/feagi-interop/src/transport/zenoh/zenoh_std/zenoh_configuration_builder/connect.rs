use serde::ser::{Serialize, SerializeMap, Serializer};

use super::error::ZenohConfigBuildError;
use super::node_mode::ZenohNodeMode;

/// Endpoints this node dials while opening a session.
///
/// Values are for the one mode chosen on [`super::node::ZenohNodeBuilder`].
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ZenohConnectConfig {
    /// How long to wait until every endpoint has connected.
    pub timeout_ms: ZenohTimeoutMs,
    /// Endpoints to dial, such as `tcp/localhost:7447`.
    pub endpoints: Vec<ZenohEndpointEntry>,
    /// Exit the process when `timeout_ms` elapses before the endpoints connect.
    pub exit_on_failure: bool,
    /// Backoff used when establishing the connection.
    pub retry: ZenohEndpointRetry,
}

impl ZenohConnectConfig {
    /// Connect settings for `mode` from the Zenoh 1.10.1 configuration example.
    pub fn for_mode(mode: ZenohNodeMode) -> Self {
        match mode {
            #[cfg(feature = "std")]
            ZenohNodeMode::Peer | ZenohNodeMode::Router => Self {
                timeout_ms: ZenohTimeoutMs::Forever,
                endpoints: Vec::new(),
                exit_on_failure: false,
                retry: ZenohEndpointRetry::default(),
            },
            ZenohNodeMode::Client => Self {
                timeout_ms: ZenohTimeoutMs::NoRetry,
                endpoints: Vec::new(),
                exit_on_failure: true,
                retry: ZenohEndpointRetry::default(),
            },
        }
    }

    /// Checks the timeout, endpoint locators, and retry bounds.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.timeout_ms.validate()?;
        for endpoint in &self.endpoints {
            endpoint.validate()?;
        }
        self.retry.validate()
    }
}

/// Endpoints other nodes can dial to reach this one.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ZenohListenConfig {
    /// How long to wait until every listen endpoint is bound.
    pub timeout_ms: ZenohTimeoutMs,
    /// Endpoints to bind.
    pub endpoints: Vec<ZenohEndpointEntry>,
    /// Exit the process when `timeout_ms` elapses before the endpoints bind.
    pub exit_on_failure: bool,
    /// Backoff used when binding a listen endpoint.
    pub retry: ZenohEndpointRetry,
}

impl ZenohListenConfig {
    /// Listen settings for `mode` from the Zenoh 1.10.1 configuration example.
    pub fn for_mode(mode: ZenohNodeMode) -> Self {
        let endpoints = match mode {
            #[cfg(feature = "std")]
            ZenohNodeMode::Router => {
                vec![ZenohEndpointEntry::Locator("tcp/[::]:7447".to_string())]
            }
            #[cfg(feature = "std")]
            ZenohNodeMode::Peer => vec![ZenohEndpointEntry::Locator("tcp/[::]:0".to_string())],
            ZenohNodeMode::Client => Vec::new(),
        };
        Self {
            timeout_ms: ZenohTimeoutMs::NoRetry,
            endpoints,
            exit_on_failure: true,
            retry: ZenohEndpointRetry::default(),
        }
    }

    /// Checks the timeout, endpoint locators, and retry bounds.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        self.timeout_ms.validate()?;
        for endpoint in &self.endpoints {
            endpoint.validate()?;
        }
        self.retry.validate()
    }
}

/// How long connect or listen waits for its endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZenohTimeoutMs {
    /// Encoded as `0`.
    NoRetry,
    /// Encoded as `-1`.
    Forever,
    /// Encoded as this positive millisecond count. Zero is [`Self::NoRetry`].
    Millis(u64),
}

impl ZenohTimeoutMs {
    /// Rejects a zero count, which is [`Self::NoRetry`].
    pub fn validate(self) -> Result<(), ZenohConfigBuildError> {
        match self {
            Self::NoRetry | Self::Forever => Ok(()),
            Self::Millis(0) => Err(ZenohConfigBuildError::ZeroMillisecondTimeout),
            Self::Millis(_) => Ok(()),
        }
    }
}

impl Serialize for ZenohTimeoutMs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::NoRetry => serializer.serialize_i64(0),
            Self::Forever => serializer.serialize_i64(-1),
            Self::Millis(milliseconds) => serializer.serialize_u64(*milliseconds),
        }
    }
}

/// One connect or listen endpoint-list entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZenohEndpointEntry {
    /// Encoded as a JSON string `<proto>/<address>[?<metadata>][#<config>]`.
    ///
    /// `prio`, `rel`, `iface`, `bind`, `dscp`, buffer sizes, and retry overrides stay in that string.
    Locator(String),
    /// Encoded as `{ strategy, locators }`. Client mode uses this to open several links to one peer.
    Group {
        strategy: ZenohEndpointGroupStrategy,
        locators: Vec<String>,
    },
}

impl ZenohEndpointEntry {
    /// Checks that a locator has a protocol and address, and that a group is non-empty.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        match self {
            Self::Locator(locator) => validate_locator(locator),
            Self::Group { locators, .. } => {
                if locators.is_empty() {
                    return Err(ZenohConfigBuildError::EmptyEndpointGroup);
                }
                for locator in locators {
                    validate_locator(locator)?;
                }
                Ok(())
            }
        }
    }
}

impl Serialize for ZenohEndpointEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Locator(locator) => serializer.serialize_str(locator),
            Self::Group { strategy, locators } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("strategy", strategy)?;
                map.serialize_entry("locators", locators)?;
                map.end()
            }
        }
    }
}

/// How a [`ZenohEndpointEntry::Group`] opens its locators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub enum ZenohEndpointGroupStrategy {
    /// Encoded as `"allOf"`.
    #[default]
    #[serde(rename = "allOf")]
    AllOf,
}

/// Backoff between connect or listen attempts.
///
/// A locator string can override these timings in its `#` config.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct ZenohEndpointRetry {
    /// Initial wait before the next attempt, in milliseconds. Default: 1000.
    pub period_init_ms: i64,
    /// Upper bound on the wait before the next attempt, in milliseconds. Default: 4000.
    pub period_max_ms: i64,
    /// Multiplier applied to the wait after each failed attempt. Default: 2.
    pub period_increase_factor: f64,
}

impl Default for ZenohEndpointRetry {
    fn default() -> Self {
        Self {
            period_init_ms: 1000,
            period_max_ms: 4000,
            period_increase_factor: 2.0,
        }
    }
}

impl ZenohEndpointRetry {
    /// Rejects a negative period, a non-positive factor, and a max below the initial wait.
    pub fn validate(self) -> Result<(), ZenohConfigBuildError> {
        if self.period_init_ms < 0 || self.period_max_ms < 0 {
            return Err(ZenohConfigBuildError::NegativeRetryPeriod);
        }
        if !(self.period_increase_factor.is_finite() && self.period_increase_factor > 0.0) {
            return Err(ZenohConfigBuildError::InvalidRetryFactor);
        }
        if self.period_max_ms < self.period_init_ms {
            return Err(ZenohConfigBuildError::RetryMaxBelowInit);
        }
        Ok(())
    }
}

/// Requires `<proto>/<address>`, where `proto` is an ASCII token.
fn validate_locator(locator: &str) -> Result<(), ZenohConfigBuildError> {
    let invalid = || ZenohConfigBuildError::InvalidEndpoint {
        locator: locator.to_string(),
    };
    let Some((proto, address)) = locator.split_once('/') else {
        return Err(invalid());
    };
    let proto_ok = !proto.is_empty() && proto.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !proto_ok || address.is_empty() {
        return Err(invalid());
    }
    Ok(())
}
