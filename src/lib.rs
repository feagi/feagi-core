// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// TODO instead of directly externing crates, we should use named modules and feature gates to more
// properly expose internals

// TODO The REST server's routing nests one async future per message it serves, so serving many
// categories exceeds the default limit of 128 when laying them out. Remove once routing no longer
// nests futures this deeply
#![recursion_limit = "512"]

pub mod runtimes;
pub mod wokr;
pub mod rt;

pub extern crate feagi_basis;
