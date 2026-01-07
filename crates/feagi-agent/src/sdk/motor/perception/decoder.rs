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
use tracing;
use std::sync::atomic::{AtomicU64, Ordering};

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
    topologies: [Option<CorticalTopology>; 3],
    tokenizer: Option<tokenizers::Tokenizer>,
}

impl PerceptionDecoder {
    /// Create a new perception decoder
    ///
    /// This method attempts to fetch topologies for all three cortical areas (oseg, oimg, oten),
    /// but will succeed even if some are missing. The decoder will only decode data for areas
    /// that exist in the topology.
    pub async fn new(
        config: PerceptionDecoderConfig,
        topology_cache: &crate::sdk::base::TopologyCache,
        tokenizer_path: Option<std::path::PathBuf>,
    ) -> Result<Self> {
        config.validate()?;

        let cortical_ids = config.cortical_ids();

        // Fetch topologies individually - allow missing areas
        // This allows the decoder to work with partial cortical area sets
        let mut topologies: [Option<CorticalTopology>; 3] = [None, None, None];
        
        for (idx, cortical_id) in cortical_ids.iter().enumerate() {
            match topology_cache.get_topology(cortical_id).await {
                Ok(topology) => {
                    topologies[idx] = Some(topology);
                }
                Err(crate::sdk::error::SdkError::TopologyNotFound(_)) => {
                    // Area doesn't exist - this is OK, decoder will skip it
                    tracing::warn!(
                        "Cortical area {} (idx {}) not found in topology - decoder will skip this area",
                        cortical_id.as_base_64(),
                        idx
                    );
                }
                Err(e) => {
                    // Other errors (network, etc.) are still failures
                    return Err(e);
                }
            }
        }
        
        // Check if at least one area exists
        if topologies.iter().all(|t| t.is_none()) {
            return Err(crate::sdk::error::SdkError::InvalidConfiguration(
                "None of the required cortical areas (oseg, oimg, oten) exist in the topology".to_string(),
            ));
        }

        // Load tokenizer if path provided
        let tokenizer = if let Some(path) = tokenizer_path {
            if !path.exists() {
                tracing::warn!(
                    "[PERCEPTION-DECODER] Tokenizer path does not exist: {:?}. oten_text will not be decoded.",
                    path
                );
                None
            } else {
                match tokenizers::Tokenizer::from_file(&path) {
                    Ok(tok) => {
                        let vocab_size = tok.get_vocab_size(true);
                        tracing::info!(
                            "[PERCEPTION-DECODER] ✅ Tokenizer loaded successfully from {:?}. Vocab size: {}",
                            path,
                            vocab_size
                        );
                        Some(tok)
                    }
                    Err(e) => {
                        tracing::error!(
                            "[PERCEPTION-DECODER] ❌ Failed to load tokenizer from {:?}: {}. oten_text will not be decoded.",
                            path,
                            e
                        );
                        return Err(SdkError::InvalidConfiguration(format!("Failed to load tokenizer: {}", e)));
                    }
                }
            }
        } else {
            tracing::warn!(
                "[PERCEPTION-DECODER] No tokenizer path provided. oten_text will not be decoded (only oten_token_id will be available)."
            );
            None
        };

        Ok(Self {
            _config: config,
            cortical_ids,
            topologies,
            tokenizer,
        })
    }

    /// Get which cortical areas are available
    /// Returns a tuple of (oseg_available, oimg_available, oten_available)
    pub fn available_areas(&self) -> (bool, bool, bool) {
        (
            self.topologies[0].is_some(),
            self.topologies[1].is_some(),
            self.topologies[2].is_some(),
        )
    }
}

impl MotorDecoder for PerceptionDecoder {
    type Output = PerceptionFrame;

    fn decode(&self, data: &CorticalMappedXYZPNeuronVoxels) -> Result<Self::Output> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Decode oseg (only if topology exists)
        let oseg = self.topologies[0].as_ref().and_then(|topo| {
            data.mappings.get(&self.cortical_ids[0]).map(|voxels| {
                let (x, y, z, p) = voxels.borrow_xyzp_vectors();
                OsegData {
                    width: topo.width,
                    height: topo.height,
                    depth: topo.depth,
                    channels: topo.channels,
                    x: x.clone(),
                    y: y.clone(),
                    z: z.clone(),
                    p: p.clone(),
                }
            })
        });

        // Decode oimg (only if topology exists)
        let oimg = self.topologies[1].as_ref().and_then(|topo| {
            data.mappings.get(&self.cortical_ids[1]).map(|voxels| {
                let (x, y, z, p) = voxels.borrow_xyzp_vectors();
                OimgData {
                    width: topo.width,
                    height: topo.height,
                    depth: topo.depth,
                    channels: topo.channels,
                    x: x.clone(),
                    y: y.clone(),
                    z: z.clone(),
                    p: p.clone(),
                }
            })
        });

        // Decode oten (only if topology exists)
        let oten_token_id = self.topologies[2].as_ref().and_then(|topo| {
            data.mappings
                .get(&self.cortical_ids[2])
                .and_then(|voxels| {
                    // Strategic debugging: summarize raw oten voxels and what the bitplane decoder
                    // "would see" at (x=0,y=0). This helps distinguish:
                    // - FEAGI motor payload not changing vs
                    // - voxels present but not at x=0,y=0 (decoder ignores them).
                    static OTEN_VOXEL_LOG_COUNT: AtomicU64 = AtomicU64::new(0);
                    let log_n = OTEN_VOXEL_LOG_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                    if log_n <= 10 || log_n % 100 == 0 {
                        let (x, y, z, p) = voxels.borrow_xyzp_vectors();
                        let depth = topo.depth;

                        let total = x.len();
                        let mut active_total: usize = 0;
                        let mut active_xy00: usize = 0;
                        let mut active_non_xy00: usize = 0;
                        let mut active_z_out_of_range: usize = 0;
                        let mut value_raw: u32 = 0;

                        // Track min/max coords for quick sanity checks.
                        let mut min_x: u32 = u32::MAX;
                        let mut max_x: u32 = 0;
                        let mut min_y: u32 = u32::MAX;
                        let mut max_y: u32 = 0;

                        for i in 0..x.len() {
                            min_x = min_x.min(x[i]);
                            max_x = max_x.max(x[i]);
                            min_y = min_y.min(y[i]);
                            max_y = max_y.max(y[i]);

                            if p[i] <= 0.0 {
                                continue;
                            }
                            active_total += 1;

                            let zi = z[i];
                            if zi >= depth {
                                active_z_out_of_range += 1;
                                continue;
                            }

                            if x[i] == 0 && y[i] == 0 {
                                active_xy00 += 1;
                                // z=0 is MSB.
                                if depth <= 32 {
                                    let weight = 1u32 << (depth - 1 - zi);
                                    value_raw |= weight;
                                }
                            } else {
                                active_non_xy00 += 1;
                            }
                        }

                        // Interpret value using TextToken "offset encoding".
                        let decoded_from_raw = if value_raw == 0 {
                            None
                        } else {
                            Some(value_raw - 1)
                        };

                        tracing::info!(
                            "[PERCEPTION-DECODER][OTEN] sample#{} total_voxels={} active_total={} active_xy00={} active_non_xy00={} active_z_out_of_range={} depth={} x_range=[{},{}] y_range=[{},{}] value_raw=0x{:X} decoded_from_raw={:?}",
                            log_n,
                            total,
                            active_total,
                            active_xy00,
                            active_non_xy00,
                            active_z_out_of_range,
                            depth,
                            if min_x == u32::MAX { 0 } else { min_x },
                            max_x,
                            if min_y == u32::MAX { 0 } else { min_y },
                            max_y,
                            value_raw,
                            decoded_from_raw
                        );
                    }

                    decode_token_id_from_xyzp_bitplanes(voxels, topo.depth)
                        .ok()
                        .flatten()
                })
        });

        let oten_text = if let (Some(token_id), Some(ref tokenizer)) =
            (oten_token_id, &self.tokenizer)
        {
            // Try decoding with skip_special_tokens=true first
            let decoded = tokenizer
                .decode(std::slice::from_ref(&token_id), true)
                .ok();
            
            // Log decoding attempts for debugging (first few and every 100th)
            static DECODE_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = DECODE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            
            if count <= 5 || count % 100 == 0 {
                match &decoded {
                    Some(text) if !text.is_empty() => {
                        tracing::info!(
                            "[PERCEPTION-DECODER] Token ID {} decoded to: \"{}\"",
                            token_id,
                            text
                        );
                    }
                    Some(text) if text.is_empty() => {
                        // Try without skip_special_tokens to see what it actually is
                        let decoded_with_special = tokenizer
                            .decode(std::slice::from_ref(&token_id), false)
                            .ok();
                        tracing::warn!(
                            "[PERCEPTION-DECODER] Token ID {} decoded to EMPTY string (skip_special=true). \
                            Without skip_special: {:?}. This might be a control/padding token.",
                            token_id,
                            decoded_with_special
                        );
                    }
                    None => {
                        tracing::error!(
                            "[PERCEPTION-DECODER] Token ID {} failed to decode! Tokenizer error.",
                            token_id
                        );
                    }
                    _ => {}
                }
            }
            
            decoded.filter(|s| !s.is_empty())
        } else {
            // Log why tokenizer isn't being used
            if oten_token_id.is_some() && self.tokenizer.is_none() {
                static WARNED_NO_TOKENIZER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !WARNED_NO_TOKENIZER.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    tracing::warn!(
                        "[PERCEPTION-DECODER] Token ID {} received but tokenizer is not loaded! \
                        oten_text will always be empty. Check tokenizer_path in controller initialization.",
                        oten_token_id.unwrap()
                    );
                }
            }
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

