// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Video encoder configuration

use crate::core::{AgentConfig, AgentType};
use crate::sdk::error::{Result, SdkError};
use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::SensoryCorticalUnit;
use std::collections::HashMap;

/// Video encoding strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoEncodingStrategy {
    /// Simple full-frame vision (iimg cortical area)
    SimpleVision,
    /// Segmented vision with gaze modulation (isvi cortical area, 9 segments)
    SegmentedVision,
}

impl VideoEncodingStrategy {
    /// Get cortical IDs for this encoding strategy
    pub fn cortical_ids(&self, unit: CorticalUnitIndex) -> Vec<CorticalID> {
        match self {
            Self::SimpleVision => SensoryCorticalUnit::get_cortical_ids_array_for_vision_with_parameters(
                FrameChangeHandling::Absolute,
                unit,
            )
            .to_vec(),
            Self::SegmentedVision => {
                SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                    FrameChangeHandling::Absolute,
                    unit,
                )
                .to_vec()
            }
        }
    }

    /// Get expected number of cortical IDs
    pub fn cortical_id_count(&self) -> usize {
        match self {
            Self::SimpleVision => 1,
            Self::SegmentedVision => 9,
        }
    }
}

/// Video encoder configuration
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::sensory::video::{VideoEncoderConfig, VideoEncodingStrategy};
///
/// let config = VideoEncoderConfig {
///     agent_id: "video-camera-01".to_string(),
///     cortical_unit_id: 0,
///     encoding_strategy: VideoEncodingStrategy::SimpleVision,
///     source_width: 640,
///     source_height: 480,
///     feagi_host: "localhost".to_string(),
///     feagi_api_port: 8080,
///     feagi_zmq_registration_port: 30001,
///     feagi_zmq_sensory_port: 5555,
///     feagi_tick_hz: 60,
///     feagi_heartbeat_interval_s: 5.0,
///     feagi_connection_timeout_ms: 5000,
///     feagi_registration_retries: 3,
///     diff_threshold: 10,
///     brightness: 0,
///     contrast: 1.0,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct VideoEncoderConfig {
    // Identity
    pub agent_id: String,
    pub cortical_unit_id: u8,  // Cortical unit index (which unit of this type)

    // Encoding strategy
    pub encoding_strategy: VideoEncodingStrategy,

    // Source frame properties
    pub source_width: u32,
    pub source_height: u32,

    // FEAGI network configuration
    pub feagi_host: String,
    pub feagi_api_port: u16,
    pub feagi_zmq_registration_port: u16,
    pub feagi_zmq_sensory_port: u16,
    pub feagi_tick_hz: u32,
    pub feagi_heartbeat_interval_s: f64,
    pub feagi_connection_timeout_ms: u64,
    pub feagi_registration_retries: u32,

    // Processing options
    pub diff_threshold: u8,
    pub brightness: i32,
    pub contrast: f32,
}

impl VideoEncoderConfig {
    /// Convert to AgentConfig for core client
    pub fn to_agent_config(&self) -> Result<AgentConfig> {
        let unit_index = CorticalUnitIndex::from(self.cortical_unit_id);

        // Get cortical IDs for this encoding strategy
        let cortical_ids = self.encoding_strategy.cortical_ids(unit_index);

        // Build cortical mappings for registration
        let mut cortical_mappings = HashMap::new();
        for (idx, id) in cortical_ids.iter().enumerate() {
            cortical_mappings.insert(id.as_base_64(), idx as u32);
        }

        let mut config = AgentConfig::new(self.agent_id.clone(), AgentType::Sensory)
            .with_registration_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_registration_port
            ))
            .with_sensory_endpoint(format!(
                "tcp://{}:{}",
                self.feagi_host, self.feagi_zmq_sensory_port
            ))
            .with_heartbeat_interval(self.feagi_heartbeat_interval_s)
            .with_connection_timeout_ms(self.feagi_connection_timeout_ms)
            .with_registration_retries(self.feagi_registration_retries)
            .with_sensory_capability(self.feagi_tick_hz as f64, None, cortical_mappings);

        // For segmented vision (9 areas), add vision capability to auto-create areas
        // The sensory mappings will still be used to register with burst engine
        if matches!(self.encoding_strategy, VideoEncodingStrategy::SegmentedVision) {
            // Use the first cortical ID as the target (segmented vision uses isvi)
            if let Some(first_id) = cortical_ids.first() {
                // Extract the area name from the cortical ID (e.g., "isvi" from bytes)
                let id_bytes = first_id.as_bytes();
                let area_name = if id_bytes[0] == b'i' {
                    // Extract area name from bytes 1-4 (e.g., "svi" -> "isvi")
                    String::from_utf8_lossy(&id_bytes[0..4.min(id_bytes.len())])
                        .trim_end_matches('\0')
                        .trim_end_matches('_')
                        .to_string()
                } else {
                    "isvi".to_string() // Default for segmented vision
                };
                
                config = config.with_vision_capability(
                    "camera",
                    (self.source_width as usize, self.source_height as usize),
                    3, // RGB channels
                    area_name,
                );
            }
        }

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.agent_id.is_empty() {
            return Err(SdkError::InvalidConfiguration(
                "agent_id cannot be empty".to_string(),
            ));
        }

        if self.source_width == 0 || self.source_height == 0 {
            return Err(SdkError::InvalidConfiguration(format!(
                "Invalid source dimensions: {}x{}",
                self.source_width, self.source_height
            )));
        }

        if self.feagi_tick_hz == 0 {
            return Err(SdkError::InvalidConfiguration(
                "feagi_tick_hz must be > 0".to_string(),
            ));
        }

        Ok(())
    }
}

