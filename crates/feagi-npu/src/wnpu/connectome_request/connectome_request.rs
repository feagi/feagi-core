use serde::{Deserialize, Serialize};
use crate::wnpu::connectome_request::sub_requests::burst_engine::{BurstEngineRequestBuilder, BurstEngineRequestEnum};
use crate::wnpu::connectome_request::sub_requests::cortical_area::CorticalAreaRequestEnum;

/// Can be passed into a Wrapped NPU to request some change to the connectome.
/// Serializable
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectomeRequest {
    enum_request: ConnectomeRequestEnum
}


/// Starting point to start building a `ConnectomeRequest`
#[derive(Debug, Clone)]
pub struct ConnectomeRequestBuilder;

impl ConnectomeRequestBuilder {
    /// Configure something related to the burst engine, such as its state or frequency
    pub fn burst_engine() -> BurstEngineRequestBuilder { BurstEngineRequestBuilder }
    
}

/// Underlying enum of `ConnectomeRequest`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum ConnectomeRequestEnum{
    /// Add / Remove / Configure Cortical Areas
    CorticalArea(CorticalAreaRequestEnum),
    /// Add / Remove / Configure Cortical Mappings
    CorticalMapping(),
    /// Init / Pause / Set Frequency of Burst Engine
    BurstEngine(BurstEngineRequestEnum),
}

impl Into<ConnectomeRequest> for ConnectomeRequestEnum
{
    fn into(self) -> ConnectomeRequest {
        ConnectomeRequest {
            enum_request: self
        }
    }
}

