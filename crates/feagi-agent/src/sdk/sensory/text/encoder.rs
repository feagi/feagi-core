//! Text encoder implementation.

use crate::core::SdkError;
use crate::sdk::base::TopologyCache;
use crate::sdk::sensory::text::config::TextEncoderConfig;
use crate::sdk::sensory::traits::SensoryEncoder;
use crate::sdk::types::{
    CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, FrameChangeHandling,
    MiscDataDimensions, SensorDeviceCache, WrappedIOData,
};
use feagi_sensorimotor::data_types::encode_token_id_to_misc_data;

/// Text encoder backed by a sensor cache.
pub struct TextEncoder {
    config: TextEncoderConfig,
    cache: SensorDeviceCache,
    cortical_ids: Vec<crate::sdk::types::CorticalID>,
    depth: u32,
}

impl TextEncoder {
    /// Create a new text encoder and register device definitions.
    pub async fn new(
        config: TextEncoderConfig,
        topology_cache: &TopologyCache,
    ) -> Result<Self, SdkError> {
        let cortical_id = config.cortical_id();
        let topology = topology_cache.get_topology(&cortical_id).await?;
        let depth = topology.depth;

        let unit = CorticalUnitIndex::from(config.cortical_unit_id);
        let channel_count =
            CorticalChannelCount::new(topology.channels).map_err(|e| {
                SdkError::Other(format!("Invalid text channel count: {e}"))
            })?;
        let dimensions = MiscDataDimensions::new(1, 1, depth)
            .map_err(|e| SdkError::Other(format!("Invalid text dimensions: {e}")))?;

        let mut cache = SensorDeviceCache::new();
        cache
            .text_english_input_register(unit, channel_count, FrameChangeHandling::Absolute, dimensions)
            .map_err(|e| SdkError::Other(format!("Text register failed: {e}")))?;

        Ok(Self {
            config,
            cache,
            cortical_ids: vec![cortical_id],
            depth,
        })
    }

    /// Return the Z depth (bitplanes) used for token encoding.
    pub fn depth(&self) -> u32 {
        self.depth
    }
}

impl SensoryEncoder for TextEncoder {
    type Input = u32;

    fn encode(&mut self, input: &Self::Input) -> Result<Vec<u8>, SdkError> {
        let misc = encode_token_id_to_misc_data(*input, self.depth)
            .map_err(|e| SdkError::Other(format!("Text encode failed: {e}")))?;

        self.cache
            .text_english_input_write(
                CorticalUnitIndex::from(self.config.cortical_unit_id),
                CorticalChannelIndex::from(0u32),
                WrappedIOData::MiscData(misc),
            )
            .map_err(|e| SdkError::Other(format!("Text cache write failed: {e}")))?;

        self.cache
            .encode_neurons_to_bytes()
            .map_err(|e| SdkError::Other(format!("Text byte encode failed: {e}")))?;

        Ok(self.cache.get_feagi_byte_container().get_byte_ref().to_vec())
    }

    fn cortical_ids(&self) -> &[crate::sdk::types::CorticalID] {
        &self.cortical_ids
    }
}
