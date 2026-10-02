//! Seeded subset draw for capped splits.
//!
//! The same `(len, keep, seed)` selects the same indices on every platform: the generator is
//! SplitMix64 (pure `u64` arithmetic, no platform RNG, no allocation beyond the index list) and
//! the shuffle is a partial Fisher-Yates with rejection sampling, so the draw is unbiased.
//! @cursor:ffi-safe

/// SplitMix64 generator (Steele, Lea, Flood 2014). Deterministic for a given seed.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(Self::GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform value in `0..bound` (`bound > 0`), rejecting the biased tail of the `u64` range.
    fn next_below(&mut self, bound: u64) -> u64 {
        let zone = u64::MAX - (u64::MAX % bound);
        loop {
            let value = self.next_u64();
            if value < zone {
                return value % bound;
            }
        }
    }
}

/// Indices of `keep` items drawn uniformly without replacement from `0..len`, ascending.
///
/// Ascending order keeps the drawn items in their dataset order, so only membership depends
/// on the seed. `keep >= len` returns every index.
pub fn draw_indices(len: usize, keep: usize, seed: u64) -> Vec<usize> {
    if keep >= len {
        return (0..len).collect();
    }
    let mut pool: Vec<usize> = (0..len).collect();
    let mut rng = SplitMix64::new(seed);
    for slot in 0..keep {
        let remaining = (len - slot) as u64;
        let pick = slot + rng.next_below(remaining) as usize;
        pool.swap(slot, pick);
    }
    let mut drawn = pool[..keep].to_vec();
    drawn.sort_unstable();
    drawn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_draws_same_indices() {
        assert_eq!(draw_indices(1000, 50, 7), draw_indices(1000, 50, 7));
    }

    #[test]
    fn different_seeds_draw_different_subsets() {
        assert_ne!(draw_indices(1000, 50, 7), draw_indices(1000, 50, 8));
    }

    #[test]
    fn draw_is_ascending_unique_and_in_range() {
        let drawn = draw_indices(500, 120, 99);
        assert_eq!(drawn.len(), 120);
        assert!(drawn.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(drawn.iter().all(|&index| index < 500));
    }

    #[test]
    fn keep_at_or_above_len_returns_everything() {
        assert_eq!(draw_indices(4, 4, 1), vec![0, 1, 2, 3]);
        assert_eq!(draw_indices(4, 10, 1), vec![0, 1, 2, 3]);
        assert!(draw_indices(0, 3, 1).is_empty());
    }

    #[test]
    fn known_vector_is_stable_across_platforms() {
        // Pinned so a generator or shuffle change is caught: recorded seeds must keep
        // reproducing the same subset.
        assert_eq!(draw_indices(10, 3, 42), KNOWN_10_3_42.to_vec());
    }

    const KNOWN_10_3_42: [usize; 3] = [2, 3, 4];

    #[test]
    fn draw_covers_the_whole_split_across_seeds() {
        let mut seen = [false; 20];
        for seed in 0..200 {
            for index in draw_indices(20, 2, seed) {
                seen[index] = true;
            }
        }
        assert!(seen.iter().all(|&hit| hit));
    }
}
