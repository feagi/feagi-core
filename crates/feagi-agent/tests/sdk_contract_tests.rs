// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Contract tests for the `feagi-agent` SDK.
//!
//! These tests serve two purposes:
//! 1) **SDK surface contract**: Ensure `feagi_agent::sdk::types` continues to export the
//!    controller-facing types that applications rely on.
//! 2) **Backend response contract (shape)**: Ensure SDK parsing remains compatible with the
//!    FEAGI HTTP API response schemas the SDK relies on (without requiring a live server).

use feagi_agent::sdk::base::TopologyCache;
use feagi_agent::sdk::types::*;

#[test]
fn sdk_types_surface_compiles() {
    // If these stop compiling, controllers will break. Keep this list small and intentional.
    let _ = CorticalUnitIndex::from(0u8);
    let _ = CorticalChannelCount::new(1).unwrap();
    let _ = CorticalChannelIndex::from(0u32);
    let _ = FrameChangeHandling::Absolute;

    // Ensure common descriptor types exist.
    let res = ImageXYResolution::new(128, 128).unwrap();
    let _props = ImageFrameProperties::new(res, ColorSpace::Gamma, ColorChannelLayout::RGB).unwrap();
}

#[test]
fn topology_parser_accepts_modern_schema() {
    // Example cortical id (iten group 0, absolute misc) just to generate a stable key.
    let id = IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute).as_io_cortical_id(
        true,
        [b't', b'e', b'n'],
        CorticalUnitIndex::from(0u8),
        CorticalSubUnitIndex::from(0u8),
    );
    let key = id.as_base_64();

    let payload = serde_json::json!({
        key.clone(): {
            "cortical_dimensions_per_device": [32, 32, 8],
            "dev_count": 3
        }
    });

    let topo = TopologyCache::parse_topology_payload(&id, &payload).unwrap();
    assert_eq!(topo.width, 32);
    assert_eq!(topo.height, 32);
    assert_eq!(topo.depth, 8);
    assert_eq!(topo.channels, 3);
}

#[test]
fn topology_parser_accepts_legacy_schema() {
    let id = IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute).as_io_cortical_id(
        true,
        [b't', b'e', b'n'],
        CorticalUnitIndex::from(0u8),
        CorticalSubUnitIndex::from(0u8),
    );
    let key = id.as_base_64();

    let payload = serde_json::json!({
        key.clone(): {
            "dimensions": [1, 1, 16],
            "dev_count": 1
        }
    });

    let topo = TopologyCache::parse_topology_payload(&id, &payload).unwrap();
    assert_eq!(topo.width, 1);
    assert_eq!(topo.height, 1);
    assert_eq!(topo.depth, 16);
    assert_eq!(topo.channels, 1);
}

