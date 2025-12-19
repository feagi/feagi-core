// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Cross-platform random number generation for FEAGI evolution.

Uses platform-specific implementations:
- Desktop/Server: `rand` crate (fast, native)
- WASM/Browser: JavaScript's `crypto.getRandomValues()` (Web Crypto API)

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

/// Generate random f64 in range [0.0, 1.0)
#[cfg(not(target_family = "wasm"))]
pub fn random_f64() -> f64 {
    use rand::Rng;
    rand::thread_rng().gen()
}

#[cfg(target_family = "wasm")]
pub fn random_f64() -> f64 {
    js_sys::Math::random()
}

/// Generate random f32 in range [0.0, 1.0)
pub fn random_f32() -> f32 {
    random_f64() as f32
}

/// Fill buffer with random bytes
#[cfg(not(target_family = "wasm"))]
pub fn random_bytes(buffer: &mut [u8]) {
    use rand::RngCore;
    rand::thread_rng().fill_bytes(buffer);
}

#[cfg(target_family = "wasm")]
pub fn random_bytes(buffer: &mut [u8]) {
    // Get browser's crypto API
    let window = web_sys::window().expect("no global window");
    let crypto = window.crypto().expect("crypto not available");

    // Fill buffer directly - web_sys supports passing &mut [u8]
    crypto
        .get_random_values_with_u8_array(buffer)
        .expect("get_random_values failed");
}

/// Generate random integer in range [min, max)
pub fn random_range(min: i32, max: i32) -> i32 {
    assert!(min < max, "min must be less than max");
    let range = (max - min) as f64;
    min + (random_f64() * range) as i32
}

/// Generate random u64 (useful for IDs, seeds, etc.)
pub fn random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    random_bytes(&mut bytes);
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_f64() {
        for _ in 0..100 {
            let val = random_f64();
            assert!(val >= 0.0 && val < 1.0, "random_f64 out of range: {}", val);
        }
    }

    #[test]
    fn test_random_f32() {
        for _ in 0..100 {
            let val = random_f32();
            assert!(val >= 0.0 && val < 1.0, "random_f32 out of range: {}", val);
        }
    }

    #[test]
    fn test_random_bytes() {
        let mut buffer = [0u8; 32];
        random_bytes(&mut buffer);

        // Extremely unlikely to get all zeros
        let all_zeros = buffer.iter().all(|&b| b == 0);
        assert!(!all_zeros, "random_bytes returned all zeros");
    }

    #[test]
    fn test_random_range() {
        for _ in 0..100 {
            let val = random_range(10, 20);
            assert!(val >= 10 && val < 20, "random_range out of range: {}", val);
        }
    }

    #[test]
    fn test_random_u64() {
        let mut values = std::collections::HashSet::new();
        for _ in 0..100 {
            let val = random_u64();
            values.insert(val);
        }

        // Should have good entropy - at least 95 unique values out of 100
        assert!(
            values.len() >= 95,
            "random_u64 has poor entropy: {} unique out of 100",
            values.len()
        );
    }
}
