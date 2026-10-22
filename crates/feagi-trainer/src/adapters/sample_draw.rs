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

/// Sample indices for a class-balanced cap.
///
/// `groups` is one list of sample indices per class, in the class order chosen by the caller
/// (lowest class id first). Each class gets as equal a share of `keep` as its size allows.
/// Leftover slots go one each to the earliest classes that still have unused samples. Within a
/// class the share is a [`draw_indices`] subset from a seed derived from `seed` and that
/// class's position, so the same inputs always select the same samples. The result is ascending.
pub fn draw_balanced_indices(groups: &[Vec<usize>], keep: usize, seed: u64) -> Vec<usize> {
    if groups.is_empty() || keep == 0 {
        return Vec::new();
    }
    let quotas = balanced_quotas(groups.iter().map(|group| group.len()).collect(), keep);
    let mut rng = SplitMix64::new(seed);
    let mut selected = Vec::with_capacity(keep);
    for (group, quota) in groups.iter().zip(quotas) {
        let class_seed = rng.next_u64();
        if quota == 0 {
            continue;
        }
        for local in draw_indices(group.len(), quota, class_seed) {
            selected.push(group[local]);
        }
    }
    selected.sort_unstable();
    selected
}

/// Per-class counts that sum to `keep` and differ by at most one, capped by `sizes`.
fn balanced_quotas(sizes: Vec<usize>, keep: usize) -> Vec<usize> {
    let mut quota = vec![0usize; sizes.len()];
    let mut room = sizes;
    let mut left = keep;
    let mut active: Vec<usize> = (0..room.len()).filter(|&index| room[index] > 0).collect();
    while left > 0 && !active.is_empty() {
        let share = left / active.len();
        let extra = left % active.len();
        if share == 0 {
            for &index in active.iter().take(left) {
                quota[index] += 1;
            }
            break;
        }
        let mut given = 0usize;
        let mut next = Vec::new();
        for (rank, &index) in active.iter().enumerate() {
            let mut want = share;
            if rank < extra {
                want += 1;
            }
            let take = want.min(room[index]);
            quota[index] += take;
            room[index] -= take;
            given += take;
            if room[index] > 0 {
                next.push(index);
            }
        }
        if given == 0 {
            break;
        }
        left -= given;
        active = next;
    }
    quota
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

    fn groups_of(sizes: &[usize]) -> Vec<Vec<usize>> {
        let mut next = 0usize;
        sizes
            .iter()
            .map(|&size| {
                let group: Vec<usize> = (next..next + size).collect();
                next += size;
                group
            })
            .collect()
    }

    fn counts_by_group(groups: &[Vec<usize>], selected: &[usize]) -> Vec<usize> {
        groups
            .iter()
            .map(|group| {
                selected
                    .iter()
                    .filter(|index| group.contains(index))
                    .count()
            })
            .collect()
    }

    #[test]
    fn balanced_draw_splits_an_even_cap_across_classes() {
        let groups = groups_of(&[50; 10]);
        let selected = draw_balanced_indices(&groups, 100, 7);
        assert_eq!(selected.len(), 100);
        assert_eq!(counts_by_group(&groups, &selected), vec![10; 10]);
        assert!(selected.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn balanced_draw_gives_leftover_slots_to_earlier_classes() {
        let groups = groups_of(&[40, 40, 40]);
        let selected = draw_balanced_indices(&groups, 5, 1);
        assert_eq!(counts_by_group(&groups, &selected), vec![2, 2, 1]);
    }

    #[test]
    fn balanced_draw_does_not_ask_a_small_class_for_more_than_it_has() {
        let groups = groups_of(&[2, 100, 100]);
        let selected = draw_balanced_indices(&groups, 30, 3);
        assert_eq!(counts_by_group(&groups, &selected), vec![2, 14, 14]);
    }

    #[test]
    fn balanced_draw_repeats_for_a_seed_and_changes_with_it() {
        let groups = groups_of(&[20, 20]);
        let first = draw_balanced_indices(&groups, 4, 1);
        let again = draw_balanced_indices(&groups, 4, 1);
        let other = draw_balanced_indices(&groups, 4, 2);
        assert_eq!(first, again);
        assert_ne!(first, other);
        assert_eq!(counts_by_group(&groups, &first), vec![2, 2]);
        assert_eq!(counts_by_group(&groups, &other), vec![2, 2]);
    }
}
