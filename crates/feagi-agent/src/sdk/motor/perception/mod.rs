// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Perception decoding for FEAGI

pub mod config;
pub mod decoder;

pub use config::PerceptionDecoderConfig;
pub use decoder::{PerceptionDecoder, PerceptionFrame};

