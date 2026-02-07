// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Re-export of agent registration helpers for use by feagi-rs and other embedders.

#[cfg(feature = "feagi-agent")]
pub use crate::endpoints::agent::auto_create_cortical_areas_from_device_registrations;
