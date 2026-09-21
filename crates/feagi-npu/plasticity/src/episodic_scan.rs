// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Sparse sliding-window match for `episodic_scan`.
//!
//! Local occupancy hashes are independent of the existing episodic ID hash.

use std::collections::{HashMap, HashSet};
use xxhash_rust::xxh64::xxh64;

/// Kernel geometry used as the scan window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanKernel {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl ScanKernel {
    pub fn voxel_count(self) -> u32 {
        self.width
            .saturating_mul(self.height)
            .saturating_mul(self.depth)
    }
}

/// Flatten an area-local voxel into a stable class-channel index.
pub fn class_channel_index(x: u32, y: u32, z: u32, width: u32, height: u32) -> u32 {
    x + y * width + z * width * height
}

/// xxHash64 of sorted relative coords per temporal frame. Not the episodic ID hash.
pub fn spatial_signature_hash(frames: &[Vec<(u32, u32, u32)>]) -> u64 {
    let mut buffer = Vec::new();
    for frame in frames {
        let mut sorted = frame.clone();
        sorted.sort_unstable();
        let len = sorted.len() as u32;
        buffer.extend_from_slice(&len.to_le_bytes());
        for (x, y, z) in sorted {
            buffer.extend_from_slice(&x.to_le_bytes());
            buffer.extend_from_slice(&y.to_le_bytes());
            buffer.extend_from_slice(&z.to_le_bytes());
        }
    }
    xxh64(&buffer, 0)
}

/// Skip the whole field scan when occupancy exceeds the configured density.
pub fn should_skip_scan(fired_count: usize, field_volume: u64, scan_skip_density: f32) -> bool {
    if field_volume == 0 {
        return true;
    }
    let density = fired_count as f64 / field_volume as f64;
    density > scan_skip_density as f64
}

/// Sparse covering windows: each fired coord votes into every kernel origin that covers it.
pub fn active_window_origins(
    fired: &[(u32, u32, u32)],
    kernel: ScanKernel,
    field_width: u32,
    field_height: u32,
    field_depth: u32,
) -> Vec<(u32, u32, u32)> {
    if kernel.width == 0 || kernel.height == 0 || kernel.depth == 0 {
        return Vec::new();
    }
    let mut origins = HashSet::new();
    let max_ox = field_width.saturating_sub(kernel.width);
    let max_oy = field_height.saturating_sub(kernel.height);
    let max_oz = field_depth.saturating_sub(kernel.depth);
    for &(x, y, z) in fired {
        let ox_start = x.saturating_sub(kernel.width.saturating_sub(1));
        let oy_start = y.saturating_sub(kernel.height.saturating_sub(1));
        let oz_start = z.saturating_sub(kernel.depth.saturating_sub(1));
        let ox_end = x.min(max_ox);
        let oy_end = y.min(max_oy);
        let oz_end = z.min(max_oz);
        for ox in ox_start..=ox_end {
            for oy in oy_start..=oy_end {
                for oz in oz_start..=oz_end {
                    origins.insert((ox, oy, oz));
                }
            }
        }
    }
    let mut out: Vec<(u32, u32, u32)> = origins.into_iter().collect();
    out.sort_unstable();
    out
}

/// Relative occupancy of one window from an absolute fired set.
pub fn window_relative_coords(
    fired: &[(u32, u32, u32)],
    origin: (u32, u32, u32),
    kernel: ScanKernel,
) -> Vec<(u32, u32, u32)> {
    let (ox, oy, oz) = origin;
    let mut rel = Vec::new();
    for &(x, y, z) in fired {
        if x >= ox
            && y >= oy
            && z >= oz
            && x < ox.saturating_add(kernel.width)
            && y < oy.saturating_add(kernel.height)
            && z < oz.saturating_add(kernel.depth)
        {
            rel.push((x - ox, y - oy, z - oz));
        }
    }
    rel.sort_unstable();
    rel
}

/// Group fired coords by burst offset for temporal stacking.
pub fn frames_to_coord_lists(frames: &[(u32, Vec<(u32, u32, u32)>)]) -> Vec<Vec<(u32, u32, u32)>> {
    frames.iter().map(|(_, coords)| coords.clone()).collect()
}

/// Build a lookup of absolute coords that fire in a frame.
pub fn fired_set(coords: &[(u32, u32, u32)]) -> HashMap<(u32, u32, u32), ()> {
    coords.iter().copied().map(|c| (c, ())).collect()
}

/// One sparse window ready for LTM spatial-signature lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanWindow {
    pub origin: (u32, u32, u32),
    pub spatial_hash: u64,
    pub activity: u32,
}

/// Collect kernel-sized windows over temporally stacked field frames.
///
/// Origins come from the newest frame. Each window is hashed from relative
/// occupancy of every temporal frame at that origin. Windows below
/// `min_window_activity` on the newest frame are dropped.
///
/// If the kernel does not fit in the field, no windows are returned.
pub fn collect_active_scan_windows(
    frames: &[Vec<(u32, u32, u32)>],
    kernel: ScanKernel,
    field_width: u32,
    field_height: u32,
    field_depth: u32,
    min_window_activity: u32,
) -> Vec<ScanWindow> {
    if kernel.width == 0
        || kernel.height == 0
        || kernel.depth == 0
        || kernel.width > field_width
        || kernel.height > field_height
        || kernel.depth > field_depth
        || frames.is_empty()
    {
        return Vec::new();
    }
    let newest = frames.last().map(Vec::as_slice).unwrap_or(&[]);
    let origins = active_window_origins(newest, kernel, field_width, field_height, field_depth);
    let mut windows = Vec::new();
    for origin in origins {
        let newest_rel = window_relative_coords(newest, origin, kernel);
        let activity = newest_rel.len() as u32;
        if activity < min_window_activity {
            continue;
        }
        let stacked: Vec<Vec<(u32, u32, u32)>> = frames
            .iter()
            .map(|frame| window_relative_coords(frame, origin, kernel))
            .collect();
        windows.push(ScanWindow {
            origin,
            spatial_hash: spatial_signature_hash(&stacked),
            activity,
        });
    }
    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_hash_is_order_independent_within_a_frame() {
        let a = spatial_signature_hash(&[vec![(0, 1, 0), (1, 0, 0)]]);
        let b = spatial_signature_hash(&[vec![(1, 0, 0), (0, 1, 0)]]);
        assert_eq!(a, b);
    }

    #[test]
    fn spatial_hash_differs_from_shifted_pattern() {
        let a = spatial_signature_hash(&[vec![(0, 0, 0)]]);
        let b = spatial_signature_hash(&[vec![(1, 0, 0)]]);
        assert_ne!(a, b);
    }

    #[test]
    fn skip_scan_uses_density_not_absolute_count() {
        assert!(!should_skip_scan(10, 100, 0.5));
        assert!(should_skip_scan(60, 100, 0.5));
        assert!(should_skip_scan(1, 0, 1.0));
    }

    #[test]
    fn sparse_windows_only_cover_active_spikes() {
        let fired = vec![(2, 2, 0)];
        let kernel = ScanKernel {
            width: 3,
            height: 3,
            depth: 1,
        };
        let origins = active_window_origins(&fired, kernel, 8, 8, 1);
        assert!(origins.contains(&(0, 0, 0)));
        assert!(origins.contains(&(2, 2, 0)));
        assert!(!origins.contains(&(5, 5, 0)));
    }

    #[test]
    fn window_relative_coords_are_kernel_local() {
        let fired = vec![(4, 5, 0), (5, 5, 0)];
        let rel = window_relative_coords(
            &fired,
            (3, 4, 0),
            ScanKernel {
                width: 3,
                height: 3,
                depth: 1,
            },
        );
        assert_eq!(rel, vec![(1, 1, 0), (2, 1, 0)]);
    }

    #[test]
    fn class_channel_index_is_row_major() {
        assert_eq!(class_channel_index(0, 0, 3, 1, 1), 3);
        assert_eq!(class_channel_index(1, 0, 0, 4, 1), 1);
    }

    #[test]
    fn collect_windows_skips_when_kernel_does_not_fit() {
        let frames = vec![vec![(0, 0, 0)]];
        let kernel = ScanKernel {
            width: 3,
            height: 1,
            depth: 1,
        };
        assert!(collect_active_scan_windows(&frames, kernel, 2, 1, 1, 1).is_empty());
    }

    #[test]
    fn collect_windows_hashes_relative_occupancy() {
        let frames = vec![vec![(4, 5, 0), (5, 5, 0)]];
        let kernel = ScanKernel {
            width: 3,
            height: 3,
            depth: 1,
        };
        let windows = collect_active_scan_windows(&frames, kernel, 8, 8, 1, 1);
        let hit = windows
            .iter()
            .find(|w| w.origin == (3, 4, 0))
            .expect("window covering the fired pair");
        let expected = spatial_signature_hash(&[vec![(1, 1, 0), (2, 1, 0)]]);
        assert_eq!(hit.spatial_hash, expected);
        assert_eq!(hit.activity, 2);
    }
}
