//! Local XYZP neuron voxel types.
//!
//! These used to live in `feagi-structures`. That crate was removed during the
//! quantized-voxel refactor, and the replacement collections in `feagi-data`
//! are not exported yet. This crate keeps the XYZP structs it actually encodes
//! and decodes, updated to current `CorticalID` / `FeagiDataError` APIs.

pub mod xyzp;
