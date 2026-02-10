// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Trivial connectivity rules - simple, non-compute-intensive morphologies.
*/

use crate::rng::get_rng;
use crate::types::Position;
use rand::Rng;

type Dimensions = (usize, usize, usize);

/// Randomizer - select random position in destination area
pub fn syn_randomizer(dst_dimensions: Dimensions) -> Position {
    let mut rng = get_rng();
    (
        rng.gen_range(0..dst_dimensions.0 as u32),
        rng.gen_range(0..dst_dimensions.1 as u32),
        rng.gen_range(0..dst_dimensions.2 as u32),
    )
}

/// Lateral pairs X - connect neurons in pairs along X axis
/// Even neurons connect to right neighbor, odd to left neighbor
pub fn syn_lateral_pairs_x(
    neuron_location: Position,
    src_dimensions: Dimensions,
) -> Option<Position> {
    let (x, y, z) = neuron_location;

    if x % 2 == 0 {
        // Even neurons connect to the right
        if (x + 1) < src_dimensions.0 as u32 {
            Some((x + 1, y, z))
        } else {
            None
        }
    } else {
        // Odd neurons connect to the left
        if x > 0 {
            Some((x - 1, y, z))
        } else {
            None
        }
    }
}

/// Last to first - connect last neuron to first (feedback connection)
pub fn syn_last_to_first(
    neuron_location: Position,
    src_dimensions: Dimensions,
) -> Option<Position> {
    let last_pos = (
        src_dimensions.0 as u32 - 1,
        src_dimensions.1 as u32 - 1,
        src_dimensions.2 as u32 - 1,
    );

    if neuron_location == last_pos {
        Some((0, 0, 0))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_randomizer() {
        let dims = (10, 10, 10);
        for _ in 0..100 {
            let pos = syn_randomizer(dims);
            assert!(pos.0 < 10);
            assert!(pos.1 < 10);
            assert!(pos.2 < 10);
        }
    }

    #[test]
    fn test_lateral_pairs() {
        // Even neuron
        let result = syn_lateral_pairs_x((2, 5, 3), (10, 10, 10));
        assert_eq!(result, Some((3, 5, 3)));

        // Odd neuron
        let result = syn_lateral_pairs_x((3, 5, 3), (10, 10, 10));
        assert_eq!(result, Some((2, 5, 3)));

        // Edge case: even at boundary (9 pairs with 8)
        let result = syn_lateral_pairs_x((8, 5, 3), (10, 10, 10));
        assert_eq!(result, Some((9, 5, 3)));

        // Edge case: even at left boundary (0 pairs with 1)
        let result = syn_lateral_pairs_x((0, 5, 3), (10, 10, 10));
        assert_eq!(result, Some((1, 5, 3)));
    }

    #[test]
    fn test_last_to_first() {
        let dims = (10, 10, 10);

        // Last position
        let result = syn_last_to_first((9, 9, 9), dims);
        assert_eq!(result, Some((0, 0, 0)));

        // Not last position
        let result = syn_last_to_first((5, 5, 5), dims);
        assert_eq!(result, None);
    }
}
