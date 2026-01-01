// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Text encoder implementation

use crate::sdk::base::CorticalTopology;
use crate::sdk::error::{Result, SdkError};
use crate::sdk::sensory::traits::SensoryEncoder;
use crate::sdk::sensory::text::config::TextEncoderConfig;
use feagi_sensorimotor::data_types::encode_token_id_to_xyzp_bitplanes;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

/// Text encoder for FEAGI sensory data
///
/// Encodes token IDs into FEAGI's bitplane XYZP format for the `iten` cortical area.
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::sensory::text::{TextEncoder, TextEncoderConfig};
/// use feagi_agent::sdk::base::TopologyCache;
///
/// // Create encoder
/// let config = TextEncoderConfig { /* ... */ };
/// let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
/// let encoder = TextEncoder::new(config, &topology_cache).await?;
///
/// // Encode tokens
/// let token_id = 123;
/// let encoded = encoder.encode(&token_id)?;
/// ```
pub struct TextEncoder {
    _config: TextEncoderConfig,
    cortical_id: CorticalID,
    topology: CorticalTopology,
}

impl TextEncoder {
    /// Create a new text encoder
    ///
    /// This fetches the iten topology from FEAGI and configures the encoder.
    ///
    /// # Arguments
    /// * `config` - Encoder configuration
    /// * `topology_cache` - Topology cache for fetching cortical dimensions
    pub async fn new(
        config: TextEncoderConfig,
        topology_cache: &crate::sdk::base::TopologyCache,
    ) -> Result<Self> {
        config.validate()?;

        let cortical_id = config.cortical_id();

        // Fetch topology
        let topology = topology_cache.get_topology(&cortical_id).await?;

        Ok(Self {
            _config: config,
            cortical_id,
            topology,
        })
    }

    /// Get the topology depth (number of bits for token encoding)
    pub fn depth(&self) -> u32 {
        self.topology.depth
    }
}

impl SensoryEncoder for TextEncoder {
    type Input = u32;

    fn encode(&mut self, token_id: &Self::Input) -> Result<Vec<u8>> {
        // Encode token ID to bitplanes
        let voxels = encode_token_id_to_xyzp_bitplanes(*token_id, self.topology.depth)
            .map_err(|e| SdkError::EncodingFailed(format!("Token encoding failed: {:?}", e)))?;

        // Wrap in mapped container
        let mut mapped = CorticalMappedXYZPNeuronVoxels::new_with_capacity(1);
        let (x, y, z, p) = voxels.borrow_xyzp_vectors();
        let target = mapped.ensure_clear_and_borrow_mut(&self.cortical_id);
        target.clear();
        target.ensure_capacity(x.len());
        target
            .update_vectors_from_external(|xv, yv, zv, pv| {
                xv.extend_from_slice(x);
                yv.extend_from_slice(y);
                zv.extend_from_slice(z);
                pv.extend_from_slice(p);
                Ok(())
            })
            .map_err(|e| SdkError::EncodingFailed(format!("XYZP update failed: {:?}", e)))?;

        // Serialize
        let mut byte_container = feagi_serialization::FeagiByteContainer::new_empty();
        byte_container
            .overwrite_byte_data_with_single_struct_data(&mapped, 0)
            .map_err(|e| SdkError::EncodingFailed(format!("Serialization failed: {:?}", e)))?;

        Ok(byte_container.get_byte_ref().to_vec())
    }

    fn cortical_ids(&self) -> &[CorticalID] {
        std::slice::from_ref(&self.cortical_id)
    }
}

