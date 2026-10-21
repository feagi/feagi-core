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
pub extern crate feagi_basis;

/*
picoserve should be spawning the right request type
what is causing all this nesting
proper http error codes
json generator should be static / schemas?
default worker spawner effort (nesting?)
a way to group channels a bit easier? (trait shenanigans?)
 */