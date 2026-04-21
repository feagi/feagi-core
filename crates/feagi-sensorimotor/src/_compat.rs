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
pub use feagi_structures::neuron_voxels::coord_potential::{CorticalMappedNeuronVoxelCoordVectors, NeuronVoxelCoordVector};

/// Legacy name for the pre-refactor per-cortical-area XYZP voxel-data container,
/// instantiated for the standard desktop path (f32 potentials, u32 coords,
/// u32 neuron-index, u16 cortical-area index).
pub type CorticalMappedXYZPNeuronVoxels =
    CorticalMappedNeuronVoxelCoordVectors<f32, u32, u32, u16>;

/// Legacy name for the pre-refactor sparse (x, y, z, potential) per-area
/// container, instantiated for the same desktop path.
pub type NeuronVoxelXYZPSparseVectors = NeuronVoxelCoordVector<f32, u32, u32>;

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
            fn from(a: $XYZ_a) -> Self { $XYZ_b::new(a.width, a.height, a.depth).unwrap() }
        }
        impl From<$XYZ_b> for $XYZ_a {
            fn from(b: $XYZ_b) -> Self { $XYZ_a::new(b.width, b.height, b.depth).unwrap() }
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
