//! Encoding-aware merge of voxel firing for one cortical area.
//!
//! One home for the rule used wherever several frames for the same area meet in one burst:
//! sensory frames from an agent (several per burst) and motor firing from skipped bursts.
//! The mode comes from the area's IO data flag, so the burst engine stays encoding-agnostic.

use crate::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use crate::genomic::cortical_area::CorticalID;
use std::collections::HashMap;

/// How several frames for one area are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoMergeMode {
    /// The latest frame's voxels replace earlier ones (absolute values, or no IO flag).
    Latest,
    /// Potentials are summed per voxel (incremental values).
    Sum,
    /// Union of voxels, keeping the larger potential (boolean / event areas).
    Union,
}

/// Merge mode for an area from its cortical id's IO data flag.
///
/// Absolute frame handling and ids without an IO flag use `Latest`, incremental uses `Sum`,
/// and flags without frame handling (boolean/event) use `Union`.
pub fn io_merge_mode_for_cortical_id(id: &CorticalID) -> IoMergeMode {
    match id.extract_io_data_flag() {
        Ok(flag) => match flag.port_encoding_fields().frame {
            Some(FrameChangeHandling::Absolute) => IoMergeMode::Latest,
            Some(FrameChangeHandling::Incremental) => IoMergeMode::Sum,
            None => IoMergeMode::Union,
        },
        Err(_) => IoMergeMode::Latest,
    }
}

/// Mutable voxel columns (x, y, z, potential) holding the firing merged so far.
pub type VoxelColumnsMut<'a> = (
    &'a mut Vec<u32>,
    &'a mut Vec<u32>,
    &'a mut Vec<u32>,
    &'a mut Vec<f32>,
);

/// Voxel columns (x, y, z, potential) of the incoming frame.
pub type VoxelColumnsRef<'a> = (&'a [u32], &'a [u32], &'a [u32], &'a [f32]);

/// Fold one frame (`new`) into the firing already collected for the same area (`pending`).
///
/// Pure function: no I/O, no locks. Voxel identity is the (x, y, z) triple. `Latest` replaces
/// `pending` with `new`. `Sum` and `Union` update matching voxels in place and append the
/// rest; `on_append(i)` is called with the index in `new` of each appended voxel so callers
/// can keep parallel per-voxel data (for example neuron ids) in step.
pub fn merge_area_voxels(
    pending: VoxelColumnsMut<'_>,
    new: VoxelColumnsRef<'_>,
    mode: IoMergeMode,
    mut on_append: impl FnMut(usize),
) {
    let (px, py, pz, pp) = pending;
    let (nx, ny, nz, np) = new;
    if mode == IoMergeMode::Latest {
        *px = nx.to_vec();
        *py = ny.to_vec();
        *pz = nz.to_vec();
        *pp = np.to_vec();
        return;
    }
    let mut index: HashMap<(u32, u32, u32), usize> = px
        .iter()
        .zip(py.iter())
        .zip(pz.iter())
        .enumerate()
        .map(|(i, ((x, y), z))| ((*x, *y, *z), i))
        .collect();
    for i in 0..nx.len() {
        let key = (nx[i], ny[i], nz[i]);
        let p = np[i];
        match index.get(&key) {
            Some(&at) => {
                pp[at] = if mode == IoMergeMode::Sum {
                    pp[at] + p
                } else {
                    pp[at].max(p)
                };
            }
            None => {
                index.insert(key, px.len());
                px.push(key.0);
                py.push(key.1);
                pz.push(key.2);
                pp.push(p);
                on_append(i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type V = (u32, u32, u32, f32);

    fn cols(v: &[V]) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>) {
        (
            v.iter().map(|a| a.0).collect(),
            v.iter().map(|a| a.1).collect(),
            v.iter().map(|a| a.2).collect(),
            v.iter().map(|a| a.3).collect(),
        )
    }

    fn merged(pending: &[V], new: &[V], mode: IoMergeMode) -> Vec<V> {
        let (mut x, mut y, mut z, mut p) = cols(pending);
        let (nx, ny, nz, np) = cols(new);
        merge_area_voxels(
            (&mut x, &mut y, &mut z, &mut p),
            (&nx, &ny, &nz, &np),
            mode,
            |_| {},
        );
        let mut out: Vec<V> = (0..x.len()).map(|i| (x[i], y[i], z[i], p[i])).collect();
        out.sort_by_key(|a| (a.0, a.1, a.2));
        out
    }

    #[test]
    fn absolute_latest_replaces() {
        let out = merged(
            &[(0, 0, 0, 1.0), (1, 0, 0, 2.0)],
            &[(2, 0, 0, 5.0)],
            IoMergeMode::Latest,
        );
        assert_eq!(out, vec![(2, 0, 0, 5.0)]);
    }

    #[test]
    fn incremental_sums_per_voxel() {
        let out = merged(
            &[(0, 0, 0, 1.0), (1, 0, 0, 2.0)],
            &[(1, 0, 0, 3.0), (2, 0, 0, 4.0)],
            IoMergeMode::Sum,
        );
        assert_eq!(out, vec![(0, 0, 0, 1.0), (1, 0, 0, 5.0), (2, 0, 0, 4.0)]);
    }

    #[test]
    fn boolean_unions_with_max_potential() {
        let out = merged(
            &[(0, 0, 0, 1.0), (1, 0, 0, 9.0)],
            &[(1, 0, 0, 3.0), (2, 0, 0, 4.0)],
            IoMergeMode::Union,
        );
        assert_eq!(out, vec![(0, 0, 0, 1.0), (1, 0, 0, 9.0), (2, 0, 0, 4.0)]);
    }

    #[test]
    fn on_append_reports_new_indices_only() {
        let (mut x, mut y, mut z, mut p) = cols(&[(0, 0, 0, 1.0)]);
        let (nx, ny, nz, np) = cols(&[(0, 0, 0, 1.0), (7, 0, 0, 2.0)]);
        let mut appended = Vec::new();
        merge_area_voxels(
            (&mut x, &mut y, &mut z, &mut p),
            (&nx, &ny, &nz, &np),
            IoMergeMode::Union,
            |i| appended.push(i),
        );
        assert_eq!(appended, vec![1]);
    }

    #[test]
    fn mode_without_io_flag_is_latest() {
        let id = CorticalID::try_from_bytes(b"cvcustom").unwrap();
        assert_eq!(io_merge_mode_for_cortical_id(&id), IoMergeMode::Latest);
    }
}
