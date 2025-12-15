// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Platform-agnostic random number generation for feagi-bdu.

Uses platform-specific implementations:
- Desktop/Server: `rand::thread_rng()` (fast, native)
- WASM/Browser: `rand::rngs::OsRng` with getrandom (Web Crypto API)
*/

use rand::Rng;

/// Get a platform-appropriate RNG instance
#[cfg(not(target_family = "wasm"))]
pub fn get_rng() -> impl Rng {
    rand::thread_rng()
}

#[cfg(target_family = "wasm")]
pub fn get_rng() -> impl Rng {
    use rand::rngs::OsRng;
    OsRng
}

