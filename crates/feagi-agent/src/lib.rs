// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # FEAGI Agent Library
//!
//! Complete toolkit for building FEAGI agents, from low-level protocol to high-level controllers.
//!
//! ## Core Module
//!
//! Low-level agent protocol implementation (always available):
//! - [`core::AgentClient`] - Connect to FEAGI, manage lifecycle
//! - [`core::AgentConfig`] - Configuration builder
//! - [`core::AgentType`] - Agent type (Sensory, Motor, Both)
//!
//! ## SDK Module (Optional)
//!
//! High-level tools for building controllers (enabled with `sdk` feature):
//! - [`sdk::base::Controller`] - Controller trait
//! - [`sdk::base::TopologyCache`] - Topology fetching and caching
//! - [`sdk::sensory::SensoryEncoder`] - Trait for encoding sensory data
//! - [`sdk::motor::MotorDecoder`] - Trait for decoding motor data
//!
//! ## Examples
//!
//! ### Using Core Only (Minimal Agent)
//!
//! ```ignore
//! use feagi_agent::core::{AgentClient, AgentConfig, AgentType};
//!
//! let config = AgentConfig::new("my-agent", AgentType::Sensory)
//!     .with_registration_endpoint("tcp://localhost:30001")
//!     .with_sensory_endpoint("tcp://localhost:5555");
//!
//! let mut client = AgentClient::new(config)?;
//! client.connect()?;
//! client.send_sensory_bytes(data)?;
//! ```
//!
//! ### Building a Controller with SDK
//!
//! ```ignore
//! use feagi_agent::sdk::sensory::video::{VideoEncoder, VideoEncoderConfig};
//! use feagi_agent::sdk::base::TopologyCache;
//! use feagi_agent::core::{AgentClient, AgentConfig};
//!
//! // Create topology cache (shared across encoders)
//! let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
//!
//! // Create video encoder
//! let encoder = VideoEncoder::new(config, &topology_cache).await?;
//!
//! // Create agent client
//! let mut client = AgentClient::new(agent_config)?;
//! client.connect()?;
//!
//! // Encode and send frames
//! let encoded = encoder.encode(&frame)?;
//! client.send_sensory_bytes(encoded)?;
//! ```

pub mod core;

// Re-export core types at top level for convenience
pub use core::{AgentClient, AgentConfig, AgentType, SdkError, Result};

// SDK module (behind feature flag)
#[cfg(feature = "sdk")]
pub mod sdk;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_imports() {
        // Verify all main types are accessible
        let _config = AgentConfig::new("test", AgentType::Sensory);
    }
}
