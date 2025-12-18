// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Synaptic computation module
//!
//! Platform-agnostic synaptic algorithms merged from feagi-synapse crate.

pub mod contribution;
pub mod weight;

pub use contribution::*;
pub use weight::*;
