// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Perception decoder implementation

use crate::sdk::base::CorticalTopology;
use crate::sdk::error::{Result, SdkError};
use crate::sdk::motor::perception::config::PerceptionDecoderConfig;
use crate::sdk::motor::traits::MotorDecoder;
use feagi_sensorimotor::data_types::decode_token_id_from_xyzp_bitplanes;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use serde::Serialize;

/// Perception frame output
#[derive(Debug, Clone, Serialize)]
pub struct PerceptionFrame {
    pub timestamp_ms: u64,
    pub oimg: Option<OimgData>,
    pub oseg: Option<OsegData>,
    pub oten_text: Option<String>,
    pub oten_token_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OimgData {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channels: u32,
    pub x: Vec<u32>,
    pub y: Vec<u32>,
    pub z: Vec<u32>,
    pub p: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OsegData {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channels: u32,
    pub x: Vec<u32>,
    pub y: Vec<u32>,
    pub z: Vec<u32>,
    pub p: Vec<f32>,
}

/// Perception decoder for motor data
pub struct PerceptionDecoder {
    _config: PerceptionDecoderConfig,
    cortical_ids: [CorticalID; 3],
    topologies: [CorticalTopology; 3],
    tokenizer: Option<tokenizers::Tokenizer>,
}

impl PerceptionDecoder {
    /// Create a new perception decoder
    pub async fn new(
        config: PerceptionDecoderConfig,
        topology_cache: &crate::sdk::base::TopologyCache,
        tokenizer_path: Option<std::path::PathBuf>,
    ) -> Result<Self> {
        config.validate()?;

        let cortical_ids = config.cortical_ids();

        // Fetch topologies
        let topologies_vec = topology_cache
            .get_topologies(&cortical_ids)
            .await?;

        let topologies: [CorticalTopology; 3] = [
            topologies_vec[0].clone(),
            topologies_vec[1].clone(),
            topologies_vec[2].clone(),
        ];

        // Load tokenizer if path provided
        let tokenizer = if let Some(path) = tokenizer_path {
            Some(
                tokenizers::Tokenizer::from_file(path)
                    .map_err(|e| SdkError::InvalidConfiguration(format!("Failed to load tokenizer: {}", e)))?,
            )
        } else {
            None
        };

        Ok(Self {
            _config: config,
            cortical_ids,
            topologies,
            tokenizer,
        })
    }
}

impl MotorDecoder for PerceptionDecoder {
    type Output = PerceptionFrame;

    fn decode(&self, data: &CorticalMappedXYZPNeuronVoxels) -> Result<Self::Output> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Decode oseg
        let oseg = data.mappings.get(&self.cortical_ids[0]).map(|voxels| {
            let (x, y, z, p) = voxels.borrow_xyzp_vectors();
            OsegData {
                width: self.topologies[0].width,
                height: self.topologies[0].height,
                depth: self.topologies[0].depth,
                channels: self.topologies[0].channels,
                x: x.clone(),
                y: y.clone(),
                z: z.clone(),
                p: p.clone(),
            }
        });

        // Decode oimg
        let oimg = data.mappings.get(&self.cortical_ids[1]).map(|voxels| {
            let (x, y, z, p) = voxels.borrow_xyzp_vectors();
            OimgData {
                width: self.topologies[1].width,
                height: self.topologies[1].height,
                depth: self.topologies[1].depth,
                channels: self.topologies[1].channels,
                x: x.clone(),
                y: y.clone(),
                z: z.clone(),
                p: p.clone(),
            }
        });

        // Decode oten
        let oten_token_id = data
            .mappings
            .get(&self.cortical_ids[2])
            .and_then(|voxels| {
                decode_token_id_from_xyzp_bitplanes(voxels, self.topologies[2].depth)
                    .ok()
                    .flatten()
            });

        let oten_text = if let (Some(token_id), Some(ref tokenizer)) =
            (oten_token_id, &self.tokenizer)
        {
            tokenizer
                .decode(std::slice::from_ref(&token_id), true)
                .ok()
                .filter(|s| !s.is_empty())
        } else {
            None
        };

        Ok(PerceptionFrame {
            timestamp_ms: now_ms,
            oseg,
            oimg,
            oten_text,
            oten_token_id,
        })
    }

    fn cortical_ids(&self) -> &[CorticalID] {
        &self.cortical_ids
    }
}

