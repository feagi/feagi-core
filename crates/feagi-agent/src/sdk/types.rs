// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//
//! SDK public types (facade / hybrid stability model)
//!
//! ## Strategy
//! FEAGI is intended for commercial use where controller code must remain stable and certifiable.
//! To achieve this, **application/controller code should depend only on `feagi-agent`**.
//!
//! The FEAGI Rust ecosystem has multiple internal crates (e.g. `feagi-structures`,
//! `feagi-sensorimotor`) that may evolve as the backend evolves. The SDK uses a **hybrid**
//! approach:
//!
//! - **Facade**: controllers import FEAGI data-model and descriptor types via
//!   `feagi_agent::sdk::types::*` instead of importing internal crates directly.
//! - **Re-exported model**: these are *real* types from the internal crates, re-exported here.
//!   This keeps the API ergonomic and zero-cost.
//! - **Stability contract**: the set of types re-exported from this module is treated as SDK API.
//!   Internal crate paths and internal module layouts can change without breaking controllers.
//!
//! If you need a type in controller code, prefer requesting it be added to this module rather
//! than importing internal crates directly.

// Sensorimotor public types used by controllers
pub use feagi_sensorimotor::caching::{MotorDeviceCache, SensorDeviceCache};
pub use feagi_sensorimotor::data_types::{
    encode_token_id_to_misc_data, ImageFrame,
};
pub use feagi_sensorimotor::data_types::descriptors::{
    ColorChannelLayout, ColorSpace, ImageFrameProperties, ImageXYResolution, MiscDataDimensions,
    SegmentedImageFrameProperties,
};
pub use feagi_sensorimotor::data_types::GazeProperties;
pub use feagi_sensorimotor::wrapped_io_data::WrappedIOData;

// Core FEAGI structures (IDs, flags, and voxel containers)
pub use feagi_structures::genomic::cortical_area::CorticalID;
pub use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex, CorticalSubUnitIndex, CorticalUnitIndex,
};
pub use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag,
};
pub use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

