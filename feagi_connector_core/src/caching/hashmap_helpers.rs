use feagi_data_structures::genomic::descriptors::{CorticalGroupIndex, CorticalChannelIndex, AgentDeviceIndex};
use feagi_data_structures::genomic::CorticalType;

/// Key needed to get direct access to device channel cache
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct FullChannelCacheKey {
    pub(crate) cortical_type: CorticalType,
    pub(crate) cortical_group: CorticalGroupIndex,
    pub(crate) channel: CorticalChannelIndex,
}

impl FullChannelCacheKey {
    pub(crate) fn new(cortical_type: CorticalType, cortical_group: CorticalGroupIndex, channel: CorticalChannelIndex) -> Self {
        FullChannelCacheKey {
            cortical_type,
            cortical_group,
            channel,
        }
    }
}



#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct CorticalAreaMetadataKey {
    pub(crate) cortical_type: CorticalType,
    pub(crate) cortical_group: CorticalGroupIndex,
}

impl CorticalAreaMetadataKey {
    pub(crate) fn new(cortical_type: CorticalType, cortical_group: CorticalGroupIndex) -> Self {
        CorticalAreaMetadataKey {
            cortical_type,
            cortical_group,
        }
    }
}


#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct AccessAgentLookupKey {
    pub(crate) cortical_type: CorticalType,
    pub(crate) agent_index: AgentDeviceIndex,
}

impl AccessAgentLookupKey {
    pub(crate) fn new(cortical_type: CorticalType, agent_index: AgentDeviceIndex) -> Self {
        AccessAgentLookupKey{
            cortical_type,
            agent_index,
        }
    }
}

