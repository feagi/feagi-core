// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Block connection morphology - identity mapping using vector (0,0,0).

Block_to_block is equivalent to vectors morphology with vector (0, 0, 0).
It reuses the vector offset logic to avoid code duplication.
*/

use crate::connectivity::rules::apply_vector_offset;
use crate::types::BduResult;

/// Block connection mapping - identity mapping (like vector (0,0,0)).
///
/// Maps source coordinates to destination coordinates with zero offset.
/// This is equivalent to vectors morphology with vector (0, 0, 0).
/// Reuses `apply_vector_offset` with vector (0, 0, 0) to avoid code duplication.
///
/// Note: scaling_factor parameter is ignored (kept for backward compatibility).
pub fn syn_block_connection(
    _src_area_id: &str,
    _dst_area_id: &str,
    neuron_location: crate::types::Position,
    _src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    _scaling_factor: u32,
) -> BduResult<crate::types::Position> {
    // Reuse vector offset logic with zero vector (0, 0, 0) for identity mapping
    apply_vector_offset(neuron_location, (0, 0, 0), 1.0, dst_dimensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_connection() {
        // Identity mapping: source coordinate maps to same destination coordinate (clamped)
        // Uses apply_vector_offset with vector (0, 0, 0)
        let result = syn_block_connection("src", "dst", (5, 5, 3), (100, 10, 10), (10, 10, 10), 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (5, 5, 3)); // Same coordinates

        // Test clamping: source coordinate larger than destination dimension
        let result =
            syn_block_connection("src", "dst", (20, 5, 3), (100, 10, 10), (10, 10, 10), 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (9, 5, 3)); // Clamped to max (10-1=9)

        // Test identity mapping at origin
        let result = syn_block_connection("src", "dst", (0, 0, 0), (10, 10, 10), (10, 10, 10), 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (0, 0, 0)); // Same coordinates
    }
}
