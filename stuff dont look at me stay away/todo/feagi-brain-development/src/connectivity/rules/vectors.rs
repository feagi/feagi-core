// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Vector-based connectivity - offset-based connection patterns.
*/

use crate::types::{BduResult, Position};

type Dimensions = (usize, usize, usize);

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Apply vector offset to source position.
///
/// Returns `None` if the offset falls outside destination dimensions.
pub fn apply_vector_offset(
    src_position: Position,
    vector: (i32, i32, i32),
    morphology_scalar: f32,
    dst_dimensions: Dimensions,
) -> Option<Position> {
    let (src_x, src_y, src_z) = src_position;
    let (vec_x, vec_y, vec_z) = vector;

    // Apply scalar
    let scaled_x = (vec_x as f32 * morphology_scalar) as i32;
    let scaled_y = (vec_y as f32 * morphology_scalar) as i32;
    let scaled_z = (vec_z as f32 * morphology_scalar) as i32;

    // Apply offset without clamping (cast to i32 for signed arithmetic)
    let dst_x = src_x as i32 + scaled_x;
    let dst_y = src_y as i32 + scaled_y;
    let dst_z = src_z as i32 + scaled_z;

    let dims_x = dst_dimensions.0 as i32;
    let dims_y = dst_dimensions.1 as i32;
    let dims_z = dst_dimensions.2 as i32;

    if dst_x < 0 || dst_x >= dims_x || dst_y < 0 || dst_y >= dims_y || dst_z < 0 || dst_z >= dims_z
    {
        return None;
    }

    let dst_x = dst_x as u32;
    let dst_y = dst_y as u32;
    let dst_z = dst_z as u32;

    Some((dst_x, dst_y, dst_z))
}

/// Apply vector offset to source position with clamping to destination dimensions.
pub(super) fn apply_vector_offset_clamped(
    src_position: Position,
    vector: (i32, i32, i32),
    morphology_scalar: f32,
    dst_dimensions: Dimensions,
) -> BduResult<Position> {
    let (src_x, src_y, src_z) = src_position;
    let (vec_x, vec_y, vec_z) = vector;

    // Apply scalar
    let scaled_x = (vec_x as f32 * morphology_scalar) as i32;
    let scaled_y = (vec_y as f32 * morphology_scalar) as i32;
    let scaled_z = (vec_z as f32 * morphology_scalar) as i32;

    // Apply offset and clamp to dimensions (cast to i32 for signed arithmetic)
    let dst_x = (src_x as i32 + scaled_x)
        .max(0)
        .min(dst_dimensions.0 as i32 - 1) as u32;
    let dst_y = (src_y as i32 + scaled_y)
        .max(0)
        .min(dst_dimensions.1 as i32 - 1) as u32;
    let dst_z = (src_z as i32 + scaled_z)
        .max(0)
        .min(dst_dimensions.2 as i32 - 1) as u32;

    Ok((dst_x, dst_y, dst_z))
}

/// Match vectors - apply vector offset to source positions
pub fn match_vectors_batch(
    src_positions: &[Position],
    vector: (i32, i32, i32),
    morphology_scalar: f32,
    dst_dimensions: Dimensions,
) -> BduResult<Vec<Position>> {
    // Parallel processing for large batches (sequential fallback for WASM)
    #[cfg(feature = "parallel")]
    let results: Vec<_> = src_positions
        .par_iter()
        .filter_map(|&src_pos| {
            apply_vector_offset(src_pos, vector, morphology_scalar, dst_dimensions)
        })
        .collect();

    #[cfg(not(feature = "parallel"))]
    let results: Vec<_> = src_positions
        .iter()
        .filter_map(|&src_pos| {
            apply_vector_offset(src_pos, vector, morphology_scalar, dst_dimensions)
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_offset() {
        let result = apply_vector_offset((5, 5, 5), (1, 0, 0), 1.0, (10, 10, 10));
        assert_eq!(result, Some((6, 5, 5)));
    }

    #[test]
    fn test_vector_offset_with_scalar() {
        let result = apply_vector_offset((5, 5, 5), (2, 2, 2), 2.0, (20, 20, 20));
        assert_eq!(result, Some((9, 9, 9)));
    }

    #[test]
    fn test_vector_offset_out_of_bounds() {
        // Should error when offset is out of bounds
        let result = apply_vector_offset((8, 8, 8), (5, 5, 5), 1.0, (10, 10, 10));
        assert_eq!(result, None);
    }

    #[test]
    fn test_vector_offset_clamped() {
        // Clamped variant should stay in bounds
        let result = apply_vector_offset_clamped((8, 8, 8), (5, 5, 5), 1.0, (10, 10, 10));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (9, 9, 9)); // Clamped to max-1
    }

    #[test]
    fn test_batch_vectors() {
        let positions = vec![(0, 0, 0), (1, 1, 1), (2, 2, 2)];

        let results = match_vectors_batch(&positions, (1, 0, 0), 1.0, (10, 10, 10));

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], (1, 0, 0));
        assert_eq!(results[1], (2, 1, 1));
        assert_eq!(results[2], (3, 2, 2));
    }

    #[test]
    fn test_batch_vectors_skips_out_of_bounds() {
        let positions = vec![(8, 0, 0), (9, 0, 0)];
        let results = match_vectors_batch(&positions, (1, 0, 0), 1.0, (10, 10, 10));
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (9, 0, 0));
    }
}
