// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Local compatibility shim for feagi-sensorimotor.
//!
//! Purpose: bridge the historic API surface this crate was written against to
//! the refactored `feagi-structures`. Specifically:
//!
//! * Restores the `define_index!`, `define_xy_coordinates!`,
//!   `define_xy_dimensions!`, `define_xyz_coordinates!`, `define_xyz_dimensions!`,
//!   `define_xyz_mapping!`, `define_xyz_dimension_range!` and
//!   `define_nonzero_count!` macros that used to live in
//!   `feagi_structures::common_macros` and were replaced by a different
//!   (quant-generic) macro family.
//! * Restores the `FeagiSignal` / `FeagiSignalIndex` event-bus types that used
//!   to live at `feagi_structures::{FeagiSignal, FeagiSignalIndex}`.
//! * Re-exports `FeagiStructuresError` under its previous name `FeagiDataError`
//!   and a few other path-renamed types (e.g. `NeuronDepth`).
//!
//! This shim is intentionally scoped to **this crate only**. Downstream crates
//! should migrate to the new `feagi-structures` generic surface directly;
//! `feagi-sensorimotor` is kept on the legacy shape here because its 15k-LOC
//! surface would require an independent design pass to migrate in-place.

// ---- Prelude ----------------------------------------------------------------
//
// Every file in this crate that relies on the restored legacy surface should
// start with:
//
// ```rust
// use crate::_compat::prelude::*;
// ```
//
// so that extension-trait methods (width/height/depth, get_neurons_of,
// borrow_xyzp_vectors, etc.) resolve at call sites.
#[doc(hidden)]
pub mod prelude {
    pub use super::LegacyChannelDimensions;
    pub use super::LegacyChannelIndexValue;
    pub use super::LegacyCorticalMappedXyzpApi;
    pub use super::LegacyNeuronVoxelXyzpApi;
    // Frequently-used descriptor types that pre-refactor code reached via
    // bare unqualified names; re-exporting here lets call sites that only
    // `use crate::_compat::prelude::*;` still compile unchanged.
    pub use feagi_structures::neuron_voxels::descriptors::NeuronVoxelPotential;
}

// ---- Error type aliases -----------------------------------------------------

/// Re-export of `feagi_structures::FeagiStructuresError` under its legacy name.
///
/// The legacy macros in this module reference `FeagiDataError` by bare path, so
/// the alias is exported at both crate root (via `pub use _compat::FeagiDataError`)
/// and from this module for macro hygiene.
pub use feagi_structures::FeagiStructuresError as FeagiDataError;

/// Legacy alias: `NeuronDepth` was renamed to `CorticalChannelNeuronDepth`.
pub use feagi_structures::genomic::cortical_area::descriptors::CorticalChannelNeuronDepth as NeuronDepth;

// ---- XYZP voxel-data type aliases ------------------------------------------
//
// The pre-refactor API exposed two concrete types:
//
// * `CorticalMappedXYZPNeuronVoxels` — map of CorticalID -> per-area voxel data.
// * `NeuronVoxelXYZPSparseVectors`   — per-area sparse (x,y,z,potential) vectors.
//
// These were replaced by generic equivalents in feagi-structures:
//
// * `CorticalMappedNeuronVoxelCoordVectors<V, C, N, A>`
// * `NeuronVoxelCoordVector<V, C, N>`
//
// feagi-sensorimotor historically assumed the `(V=f32, C=u32, N=u32, A=u16)`
// desktop/std instantiation, which matches `feagi_npu_structures::StdNPUQuantization`
// and `feagi-npu-burst-engine`. We expose that instantiation under the legacy
// names so call sites don't need to be re-parameterised.
pub use feagi_structures::neuron_voxels::coord_potential::{
    CorticalMappedNeuronVoxelCoordVectors, NeuronVoxelCoordVector, NeuronVoxelXYZP,
};

/// Legacy name for the pre-refactor per-cortical-area XYZP voxel-data container,
/// instantiated for the standard desktop path (f32 potentials, u32 coords,
/// u32 neuron-index, u16 cortical-area index).
pub type CorticalMappedXYZPNeuronVoxels =
    CorticalMappedNeuronVoxelCoordVectors<f32, u32, u32, u16>;

/// Legacy name for the pre-refactor sparse (x, y, z, potential) per-area
/// container, instantiated for the same desktop path.
pub type NeuronVoxelXYZPSparseVectors = NeuronVoxelCoordVector<f32, u32, u32>;

// ---- Legacy method shims: CorticalMappedNeuronVoxelCoordVectors -------------
//
// The pre-refactor `CorticalMappedXYZPNeuronVoxels` exposed a method surface
// (`get_neurons_of`, `ensure_clear_and_borrow_mut`, `clear_neurons_only`) that
// encoders/decoders in feagi-sensorimotor rely on extensively. The new
// `CorticalMappedNeuronVoxelCoordVectors<V, C, N, A>` surface uses `.get`,
// `.get_mut`, `.insert`, `.iter_mut` instead. Rather than rewrite ~20 files,
// we expose an extension trait that re-introduces the old names as thin
// adapters. Scoped to this crate only; downstream code should use the new
// API directly.

use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::descriptors::{
    NeuronVoxelDimensions, NeuronVoxelPotential,
};
use feagi_structures::neuron_voxels::traits::{
    SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionSparse,
};

/// Extension trait restoring the pre-refactor surface of
/// `CorticalMappedXYZPNeuronVoxels`.
///
/// Implemented *only* for the std/desktop instantiation
/// `CorticalMappedNeuronVoxelCoordVectors<f32, u32, u32, u16>`.
pub trait LegacyCorticalMappedXyzpApi {
    /// Read access to a cortical area's voxel collection. Returns `None` when
    /// the area is not present (matches the old contract, which was used by
    /// decoders to short-circuit when an area hadn't produced data this tick).
    fn get_neurons_of(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<&NeuronVoxelXYZPSparseVectors>;

    /// Ensures an entry exists for `cortical_id`, clears any previous voxel
    /// data, and returns a mutable borrow. If the area is new, it is created
    /// with a `(1, 1, 1)` placeholder dimension — dimensions are not carried
    /// in the legacy wire format, and the encoders that use this method
    /// drive coordinates directly from the pipeline so the dim field is
    /// never read on the write path. The deserialization contract
    /// documented on `CorticalMappedNeuronVoxelCoordVectors` still applies:
    /// if callers plan to round-trip this through serialization they must
    /// set proper dimensions via `insert`.
    fn ensure_clear_and_borrow_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> &mut NeuronVoxelXYZPSparseVectors;

    /// Clears all per-area voxel data in place, keeping the cortical-area
    /// mapping entries (and their dimensions) intact.
    fn clear_neurons_only(&mut self);
}

impl LegacyCorticalMappedXyzpApi for CorticalMappedXYZPNeuronVoxels {
    #[inline]
    fn get_neurons_of(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<&NeuronVoxelXYZPSparseVectors> {
        self.get(cortical_id)
    }

    fn ensure_clear_and_borrow_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> &mut NeuronVoxelXYZPSparseVectors {
        if !self.contains_key(cortical_id) {
            let placeholder_dims = NeuronVoxelDimensions::<u32>::new(1, 1, 1)
                .expect("(1,1,1) is a valid non-zero dimension");
            let empty = NeuronVoxelCoordVector::<f32, u32, u32>::new(placeholder_dims, 0u32);
            self.insert(*cortical_id, empty);
        } else {
            let entry = self
                .get_mut(cortical_id)
                .expect("contains_key was just checked");
            entry.clear_all_neurons();
        }
        self.get_mut(cortical_id)
            .expect("inserted or pre-existing entry must be present")
    }

    fn clear_neurons_only(&mut self) {
        for (_id, area) in self.iter_mut() {
            area.clear_all_neurons();
        }
    }
}

// ---- Legacy method shims: NeuronVoxelCoordVector ----------------------------
//
// Restores `borrow_xyzp_vectors`, `new_from_vectors`, `len`, `clear`,
// `ensure_capacity`, and `update_vectors_from_external` on the std/desktop
// instantiation. These are implemented on top of the additive SoA accessors
// added to `feagi_structures::neuron_voxels::coord_potential::NeuronVoxelCoordVector`
// (`coord_x_slice` / `coord_y_slice` / `coord_z_slice` / `potentials_slice`,
// `with_parts_mut`, `from_parts`).

pub trait LegacyNeuronVoxelXyzpApi {
    /// Returns SoA slices over the underlying (x, y, z, potential) vectors.
    /// Potentials are exposed as raw `f32` rather than `NeuronVoxelPotential<f32>`
    /// to match the pre-refactor signature consumed by encoders.
    fn borrow_xyzp_vectors(&self) -> (&[u32], &[u32], &[u32], &[f32]);

    /// Total number of neuron voxels currently stored.
    fn len(&self) -> usize;

    /// True when no voxels are stored.
    fn is_empty(&self) -> bool;

    /// Iterator over `NeuronVoxelXYZP<f32, u32>` items. Legacy decoders
    /// consume `neuron.coordinate.{x,y,z}` and `neuron.potential` off the
    /// yielded items.
    fn iter(
        &self,
    ) -> Box<
        dyn Iterator<
                Item = feagi_structures::neuron_voxels::coord_potential::NeuronVoxelXYZP<f32, u32>,
            > + '_,
    >;

    /// Pushes a single `(x, y, z, potential)` voxel without bounds-checking
    /// against the per-area dimensions. This matches the pre-refactor
    /// `push_raw` on `NeuronVoxelXYZPSparseVectors` that encoders rely on
    /// for per-channel fan-out.
    fn push_raw(&mut self, x: u32, y: u32, z: u32, potential: f32);

    /// Clears all voxels (dimensions are preserved).
    fn clear(&mut self);

    /// Reserves capacity for `n` additional voxels.
    fn ensure_capacity(&mut self, n: usize);

    /// Runs `f` with mutable borrows of the (x, y, z, potential) SoA vectors.
    /// Matches the pre-refactor signature: the closure returns
    /// `Result<(), FeagiDataError>` and the outer method propagates the
    /// same. Potentials are exposed as `Vec<f32>` via a transparent
    /// reinterpretation of `Vec<NeuronVoxelPotential<f32>>`.
    fn update_vectors_from_external<F>(&mut self, f: F) -> Result<(), FeagiDataError>
    where
        F: FnOnce(
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<f32>,
        ) -> Result<(), FeagiDataError>;

    /// Legacy constructor: builds a sparse per-area voxel collection from
    /// four parallel vectors. A `(1, 1, 1)` placeholder dimension is used
    /// (see rationale on `LegacyCorticalMappedXyzpApi::ensure_clear_and_borrow_mut`).
    fn new_from_vectors(
        x: Vec<u32>,
        y: Vec<u32>,
        z: Vec<u32>,
        p: Vec<f32>,
    ) -> NeuronVoxelXYZPSparseVectors;
}

/// Legacy 0-arg constructor replacement for `NeuronVoxelXYZPSparseVectors::new()`
/// from the pre-refactor API. The new type's inherent `new(dims, count)`
/// takes mandatory parameters, so call sites that relied on the empty
/// constructor (`vec![...; N]`, placeholder scratch buffers, etc.) use this
/// shim instead. Produces an empty collection with a `(1, 1, 1)` placeholder
/// dimension and zero pre-allocation.
#[inline]
pub fn empty_sparse_vectors() -> NeuronVoxelXYZPSparseVectors {
    let dims = NeuronVoxelDimensions::<u32>::new(1, 1, 1)
        .expect("(1,1,1) is a valid non-zero dimension");
    NeuronVoxelCoordVector::<f32, u32, u32>::new(dims, 0u32)
}

impl LegacyNeuronVoxelXyzpApi for NeuronVoxelXYZPSparseVectors {
    #[inline]
    fn borrow_xyzp_vectors(&self) -> (&[u32], &[u32], &[u32], &[f32]) {
        let potentials_slice = self.potentials_slice();
        // NeuronVoxelPotential<f32> is #[repr(transparent)] over f32 (see
        // feagi-structures::neuron_voxels::descriptors). We reinterpret the
        // slice to hand callers the raw-potential shape they expect.
        //
        // SAFETY: NeuronVoxelPotential<f32> is declared with
        // `#[repr(transparent)]` wrapping exactly one `f32` field, and the
        // length/pointer of the resulting slice are preserved.
        let potentials_raw: &[f32] = unsafe {
            core::slice::from_raw_parts(
                potentials_slice.as_ptr() as *const f32,
                potentials_slice.len(),
            )
        };
        (
            self.coord_x_slice(),
            self.coord_y_slice(),
            self.coord_z_slice(),
            potentials_raw,
        )
    }

    #[inline]
    fn len(&self) -> usize {
        self.get_number_neuron_voxel_contained_count() as usize
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(
        &self,
    ) -> Box<
        dyn Iterator<
                Item = feagi_structures::neuron_voxels::coord_potential::NeuronVoxelXYZP<f32, u32>,
            > + '_,
    > {
        Box::new(self.iter_coordinate().map(|(coord, potential)| {
            feagi_structures::neuron_voxels::coord_potential::NeuronVoxelXYZP {
                coordinate: coord,
                potential,
            }
        }))
    }

    #[inline]
    fn push_raw(&mut self, x: u32, y: u32, z: u32, potential: f32) {
        // The coordinate is not validated against the (possibly-placeholder)
        // per-area dimensions — this mirrors the pre-refactor contract.
        self.push_neuron_voxel_unchecked(
            feagi_structures::neuron_voxels::descriptors::NeuronVoxelCoordinate::new(x, y, z),
            feagi_structures::neuron_voxels::descriptors::NeuronVoxelPotential::from(potential),
        );
    }

    #[inline]
    fn clear(&mut self) {
        self.clear_all_neurons();
    }

    #[inline]
    fn ensure_capacity(&mut self, n: usize) {
        self.reserve(n as u32);
    }

    fn update_vectors_from_external<F>(&mut self, f: F) -> Result<(), FeagiDataError>
    where
        F: FnOnce(
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<f32>,
        ) -> Result<(), FeagiDataError>,
    {
        let mut outcome: Result<(), FeagiDataError> = Ok(());
        self.with_parts_mut(|x, y, z, p| {
            // Reinterpret Vec<NeuronVoxelPotential<f32>> as Vec<f32> for the
            // duration of the closure. NeuronVoxelPotential<f32> is
            // #[repr(transparent)] over f32, so the layout is identical.
            //
            // SAFETY: transparent wrapper over f32; we restore the Vec into
            // its original shape before returning by forgetting the
            // reinterpreted vector and re-materialising the original from
            // the same (ptr, len, cap) triple.
            let cap = p.capacity();
            let len = p.len();
            let ptr = p.as_mut_ptr() as *mut f32;
            // Pull the original vector out so the reinterpreted Vec<f32>
            // is the sole owner for the duration of the closure.
            let original = core::mem::replace(p, Vec::new());
            core::mem::forget(original);
            let mut raw_p: Vec<f32> = unsafe { Vec::from_raw_parts(ptr, len, cap) };
            outcome = f(x, y, z, &mut raw_p);
            // Recover the typed vector from the (possibly-reallocated)
            // raw vector.
            let new_len = raw_p.len();
            let new_cap = raw_p.capacity();
            let new_ptr = raw_p.as_mut_ptr() as *mut NeuronVoxelPotential<f32>;
            core::mem::forget(raw_p);
            let restored: Vec<NeuronVoxelPotential<f32>> =
                unsafe { Vec::from_raw_parts(new_ptr, new_len, new_cap) };
            *p = restored;
        });
        outcome
    }

    fn new_from_vectors(
        x: Vec<u32>,
        y: Vec<u32>,
        z: Vec<u32>,
        p: Vec<f32>,
    ) -> NeuronVoxelXYZPSparseVectors {
        let placeholder_dims = NeuronVoxelDimensions::<u32>::new(1, 1, 1)
            .expect("(1,1,1) is a valid non-zero dimension");
        // Reinterpret Vec<f32> as Vec<NeuronVoxelPotential<f32>> via
        // the transparent-wrapper pun.
        //
        // SAFETY: NeuronVoxelPotential<f32> is #[repr(transparent)] over f32.
        let mut p_vec = p;
        let len = p_vec.len();
        let cap = p_vec.capacity();
        let ptr = p_vec.as_mut_ptr() as *mut NeuronVoxelPotential<f32>;
        core::mem::forget(p_vec);
        let p_typed: Vec<NeuronVoxelPotential<f32>> =
            unsafe { Vec::from_raw_parts(ptr, len, cap) };
        NeuronVoxelCoordVector::<f32, u32, u32>::from_parts(placeholder_dims, x, y, z, p_typed)
            .expect("SoA vector lengths must match")
    }
}

// ---- CorticalChannelDimensionsType width/height/depth shim ------------------
//
// Pre-refactor call sites read `.width` / `.height` / `.depth` as `u32`.
// The new type uses `.x` / `.y` / `.z` each wrapped in `NonzeroCount<u32>`.
// Expose a thin extension trait with the old field-like getters.

// The generic `CorticalChannel*Type<T>` aliases are defined inside a private
// `mod generated { ... }` block of `feagi_structures`; only the `u32`
// instantiations are re-exported. That's fine — every sensorimotor call site
// uses the `u32` alias, so we implement the compat traits against those
// aliases directly.
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelDimensions, CorticalChannelIndex,
};

pub trait LegacyChannelDimensions {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depth(&self) -> u32;
}

impl LegacyChannelDimensions for CorticalChannelDimensions {
    #[inline]
    fn width(&self) -> u32 {
        self.x.get()
    }
    #[inline]
    fn height(&self) -> u32 {
        self.y.get()
    }
    #[inline]
    fn depth(&self) -> u32 {
        self.z.get()
    }
}

// ---- CorticalChannelIndex / CorticalChannelCount value accessors ------------
//
// Pre-refactor these were thin `u32` wrappers implementing `Deref<Target=u32>`.
// The new versions wrap `T` (or `NonzeroCount<T>`) and require an explicit
// accessor. Provide a `.value() -> u32` method to match old usage patterns
// concisely without trying to re-implement `Deref`.

pub trait LegacyChannelIndexValue {
    fn value(&self) -> u32;
    /// Legacy alias for [`Self::value`]; pre-refactor code accessed the
    /// inner payload via `.get()` on both `CorticalChannelIndex` and
    /// `CorticalChannelCount`.
    #[inline]
    fn get(&self) -> u32 {
        self.value()
    }
}

impl LegacyChannelIndexValue for CorticalChannelIndex {
    #[inline]
    fn value(&self) -> u32 {
        self.0
    }
}

impl LegacyChannelIndexValue for CorticalChannelCount {
    #[inline]
    fn value(&self) -> u32 {
        (*self).get()
    }
}

// ---- FeagiSignal / FeagiSignalIndex (event bus) -----------------------------

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// Legacy `FeagiSignalIndex`. Locally defined because the host macro
/// `define_index!` is *itself* defined in this module; declaring the struct by
/// hand avoids a bootstrapping ordering hazard.
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
    serde::Serialize, serde::Deserialize,
)]
pub struct FeagiSignalIndex(u32);

impl FeagiSignalIndex {
    pub const fn from(var: u32) -> Self { Self(var) }
    pub const fn get(&self) -> u32 { self.0 }
}

impl std::ops::Deref for FeagiSignalIndex {
    type Target = u32;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<u32> for FeagiSignalIndex {
    fn from(value: u32) -> Self { FeagiSignalIndex(value) }
}

impl From<FeagiSignalIndex> for u32 {
    fn from(value: FeagiSignalIndex) -> Self { value.0 }
}

impl std::fmt::Display for FeagiSignalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type SignalListener<T> = Box<dyn FnMut(&T) + Send>;

/// Godot-style event signal: subscribe callbacks, emit events.
pub struct FeagiSignal<T> {
    listeners: HashMap<FeagiSignalIndex, SignalListener<T>>,
    next_index: u32,
}

impl<T> FeagiSignal<T> {
    pub fn new() -> Self {
        Self { listeners: HashMap::new(), next_index: 0 }
    }

    pub fn connect<F>(&mut self, f: F) -> FeagiSignalIndex
    where
        F: FnMut(&T) + Send + 'static,
    {
        self.listeners.insert(self.next_index.into(), Box::new(f));
        self.next_index += 1;
        (self.next_index - 1).into()
    }

    pub fn disconnect(&mut self, index: FeagiSignalIndex) -> Result<(), FeagiDataError> {
        if self.listeners.remove(&index).is_some() {
            return Ok(());
        }
        Err(FeagiDataError::BadParameters(format!(
            "No subscription found with identifier {}!",
            index
        )))
    }

    pub fn emit(&mut self, value: &T) {
        for f in self.listeners.values_mut() {
            f(value);
        }
    }

    pub fn connect_with_shared_state<S, F>(
        &mut self,
        state: Arc<Mutex<S>>,
        mut callback: F,
    ) -> FeagiSignalIndex
    where
        S: Send + 'static,
        F: FnMut(&mut S, &T) + Send + 'static,
    {
        self.connect(move |event| {
            if let Ok(mut guard) = state.lock() {
                callback(&mut *guard, event);
            }
        })
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    pub fn disconnect_all(&mut self) {
        self.listeners.clear();
    }
}

impl<T> Default for FeagiSignal<T> {
    fn default() -> Self { Self::new() }
}

impl<T> Debug for FeagiSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeagiSignal")
            .field("listener_count", &self.listeners.len())
            .field("next_index", &self.next_index)
            .field("listener_indices", &self.listeners.keys().collect::<Vec<_>>())
            .finish()
    }
}

// ---- Legacy macros ----------------------------------------------------------
//
// NOTE: These macros reference `FeagiDataError` by absolute path
// (`$crate::_compat::FeagiDataError`) so they stay hygienic regardless of
// where the invoking file is located within this crate.

/// Legacy `define_index!` — strongly-typed u32-backed index wrapper.
#[macro_export]
macro_rules! define_index {
    ($name:ident, $inner:ty, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
            serde::Serialize, serde::Deserialize,
        )]
        pub struct $name($inner);

        impl $name {
            pub const fn from(var: $inner) -> Self { Self(var) }
            pub const fn get(&self) -> $inner { self.0 }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self { $name(value) }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self { value.0 }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// Legacy `define_nonzero_count!` — non-zero integer wrapper with validation.
#[macro_export]
macro_rules! define_nonzero_count {
    ($name:ident, $base:ty, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
            serde::Serialize, serde::Deserialize,
        )]
        pub struct $name { value: $base }

        impl $name {
            pub fn new(value: $base) -> Result<Self, $crate::_compat::FeagiDataError> {
                if value == 0 {
                    return Err($crate::_compat::FeagiDataError::BadParameters(
                        "Count cannot be zero!".into(),
                    ));
                }
                Ok($name { value })
            }
        }
        impl TryFrom<$base> for $name {
            type Error = $crate::_compat::FeagiDataError;
            fn try_from(value: $base) -> Result<Self, Self::Error> { $name::new(value) }
        }
        impl From<$name> for $base {
            fn from(value: $name) -> $base { value.value }
        }
        impl std::ops::Deref for $name {
            type Target = $base;
            fn deref(&self) -> &Self::Target { &self.value }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { self.value.fmt(f) }
        }
    };
}

/// Legacy `define_xy_coordinates!` — 2D coordinate with `pub x, y` fields.
#[macro_export]
macro_rules! define_xy_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {
        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, serde::Serialize, serde::Deserialize)]
        pub struct $name { pub x: $var_type, pub y: $var_type }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Self { Self { x, y } }
        }
        impl From<$name> for ($var_type, $var_type) {
            fn from(value: $name) -> Self { (value.x, value.y) }
        }
        impl From<($var_type, $var_type)> for $name {
            fn from(value: ($var_type, $var_type)) -> Self { $name::new(value.0, value.1) }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {})", $friendly_name, self.x, self.y)
            }
        }
    };
}

/// Legacy `define_xy_dimensions!` — 2D dimension with `pub width, height` and zero-rejecting ctor.
#[macro_export]
macro_rules! define_xy_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {
        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy, Hash, Eq, serde::Serialize, serde::Deserialize)]
        pub struct $name { pub width: $var_type, pub height: $var_type }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Result<Self, $crate::_compat::FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(format!(
                        "Value cannot be {:?} in a {:?}!",
                        $invalid_zero_value, $friendly_name
                    )));
                }
                Ok(Self { width: x, height: y })
            }

            // Method forms, provided alongside the pub fields so uniform
            // `.width()` / `.height()` syntax works on every *Dimensions
            // type inside feagi-sensorimotor (the legacy macro-generated
            // structs and `CorticalChannelDimensionsType<u32>` shim both).
            #[inline] pub fn width(&self) -> $var_type { self.width }
            #[inline] pub fn height(&self) -> $var_type { self.height }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}>", $friendly_name, self.width, self.height)
            }
        }
        impl From<$name> for ($var_type, $var_type) {
            fn from(value: $name) -> Self { (value.width, value.height) }
        }
        impl TryFrom<($var_type, $var_type)> for $name {
            type Error = $crate::_compat::FeagiDataError;
            fn try_from(value: ($var_type, $var_type)) -> Result<Self, Self::Error> {
                if value.0 == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(
                        "X value cannot be zero!".into(),
                    ));
                }
                if value.1 == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(
                        "Y value cannot be zero!".into(),
                    ));
                }
                Ok(Self { width: value.0, height: value.1 })
            }
        }
    };
}

/// Legacy `define_xyz_coordinates!` — 3D coordinate with `pub x, y, z` fields.
#[macro_export]
macro_rules! define_xyz_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {
        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, serde::Serialize, serde::Deserialize)]
        pub struct $name { pub x: $var_type, pub y: $var_type, pub z: $var_type }

        impl $name {
            pub fn new(x: $var_type, y: $var_type, z: $var_type) -> Self { Self { x, y, z } }
        }
        impl From<$name> for ($var_type, $var_type, $var_type) {
            fn from(v: $name) -> Self { (v.x, v.y, v.z) }
        }
        impl From<($var_type, $var_type, $var_type)> for $name {
            fn from(v: ($var_type, $var_type, $var_type)) -> Self { $name::new(v.0, v.1, v.2) }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {}, {})", $friendly_name, self.x, self.y, self.z)
            }
        }
    };
}

/// Legacy `define_xyz_dimensions!` — 3D dimension with zero-rejecting ctor and helpers.
#[macro_export]
macro_rules! define_xyz_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {
        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub width: $var_type,
            pub height: $var_type,
            pub depth: $var_type,
        }

        impl $name {
            pub fn new(
                x: $var_type, y: $var_type, z: $var_type,
            ) -> Result<Self, $crate::_compat::FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value || z == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(format!(
                        "Value cannot be {:?} in a {:?}!",
                        $invalid_zero_value, $friendly_name
                    )));
                }
                Ok(Self { width: x, height: y, depth: z })
            }
            pub fn from_tuple(
                tuple: ($var_type, $var_type, $var_type),
            ) -> Result<Self, $crate::_compat::FeagiDataError> {
                Self::new(tuple.0, tuple.1, tuple.2)
            }
            pub fn to_tuple(&self) -> ($var_type, $var_type, $var_type) {
                (self.width, self.height, self.depth)
            }
            pub fn number_elements(&self) -> $var_type {
                self.width * self.height * self.depth
            }
            pub fn volume(&self) -> $var_type { self.number_elements() }
            pub fn total_voxels(&self) -> $var_type { self.number_elements() }
            pub fn contains(&self, pos: ($var_type, $var_type, $var_type)) -> bool {
                pos.0 < self.width && pos.1 < self.height && pos.2 < self.depth
            }

            // Method-form accessors (see rationale on the xy macro above).
            #[inline] pub fn width(&self) -> $var_type { self.width }
            #[inline] pub fn height(&self) -> $var_type { self.height }
            #[inline] pub fn depth(&self) -> $var_type { self.depth }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}, {}>", $friendly_name, self.width, self.height, self.depth)
            }
        }
        impl From<$name> for ($var_type, $var_type, $var_type) {
            fn from(v: $name) -> Self { (v.width, v.height, v.depth) }
        }
        impl TryFrom<($var_type, $var_type, $var_type)> for $name {
            type Error = $crate::_compat::FeagiDataError;
            fn try_from(value: ($var_type, $var_type, $var_type)) -> Result<Self, Self::Error> {
                if value.0 == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(
                        "X value cannot be zero!".into(),
                    ));
                }
                if value.1 == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(
                        "Y value cannot be zero!".into(),
                    ));
                }
                if value.2 == $invalid_zero_value {
                    return Err($crate::_compat::FeagiDataError::BadParameters(
                        "Z value cannot be zero!".into(),
                    ));
                }
                Ok(Self { width: value.0, height: value.1, depth: value.2 })
            }
        }
    };
}

/// Legacy `define_xyz_mapping!` — bidirectional `From` between two `define_xyz_dimensions!` types.
#[macro_export]
macro_rules! define_xyz_mapping {
    ($XYZ_a:ident, $XYZ_b:ident) => {
        impl From<$XYZ_a> for $XYZ_b {
            fn from(a: $XYZ_a) -> Self {
                // Method-form accessors so the mapping works uniformly for
                // both the legacy macro-generated dimension types (which
                // expose both fields AND methods) and
                // `feagi_structures::CorticalChannelDimensions` (which
                // gains width/height/depth via the `LegacyChannelDimensions`
                // extension trait in this module).
                use $crate::_compat::LegacyChannelDimensions as _;
                $XYZ_b::new(a.width(), a.height(), a.depth()).unwrap()
            }
        }
        impl From<$XYZ_b> for $XYZ_a {
            fn from(b: $XYZ_b) -> Self {
                use $crate::_compat::LegacyChannelDimensions as _;
                $XYZ_a::new(b.width(), b.height(), b.depth()).unwrap()
            }
        }
    };
}

/// Legacy `define_xyz_dimension_range!` — axis-aligned 3D range container.
#[macro_export]
macro_rules! define_xyz_dimension_range {
    ($name:ident, $var_type:ty, $coordinate_type:ty, $friendly_name:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub width: std::ops::Range<$var_type>,
            pub height: std::ops::Range<$var_type>,
            pub depth: std::ops::Range<$var_type>,
        }

        impl $name {
            pub fn new(
                x: std::ops::Range<$var_type>,
                y: std::ops::Range<$var_type>,
                z: std::ops::Range<$var_type>,
            ) -> Result<Self, $crate::_compat::FeagiDataError> {
                Ok($name { width: x, height: y, depth: z })
            }
            pub fn verify_coordinate_within_range(
                &self,
                coordinate: &$coordinate_type,
            ) -> Result<(), $crate::_compat::FeagiDataError> {
                if self.width.contains(&coordinate.width)
                    && self.height.contains(&coordinate.height)
                    && self.depth.contains(&coordinate.depth)
                {
                    return Ok(());
                }
                Err($crate::_compat::FeagiDataError::BadParameters(format!(
                    "Coordinate {:?} is not contained by this given range of {:?}!",
                    coordinate, self
                )))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f, "{}<{:?}, {:?}, {:?}>",
                    $friendly_name, self.width, self.height, self.depth
                )
            }
        }
    };
}
