// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Video encoding for FEAGI

pub mod config;
pub mod encoder;

pub use config::{VideoEncoderConfig, VideoEncodingStrategy};
pub use encoder::VideoEncoder;
