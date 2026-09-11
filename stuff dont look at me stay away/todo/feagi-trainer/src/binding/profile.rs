//! Binding profiles — the resolved, pinned mapping between a dataset's I/O and FEAGI areas.
//!
//! A profile names the cortical_area area(s) a selector targets plus the scheme/shape used. It is
//! produced when a run's binding is resolved and is recorded in provenance so a benchmark is
//! reproducible. The concrete cortical_area-area identifiers come from the pinned genome/connectome
//! (a design artifact), not from the Trainer.

use serde::{Deserialize, Serialize};

use crate::binding::encoding_scheme::EncodingScheme;

/// How an encoder selector maps sample features onto a FEAGI sensory (IPU) area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncoderBindingProfile {
    /// Identifier of the target sensory cortical_area area (from the pinned genome).
    pub cortical_area_id: String,
    /// Number of channels (e.g. one per scalar feature).
    pub channels: u32,
    /// The neural encoding scheme to apply.
    pub scheme: EncodingScheme,
}

/// How a decoder selector reads a FEAGI motor (OPU) area into a typed prediction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecoderBindingProfile {
    /// Identifier of the source motor cortical_area area (from the pinned genome).
    pub cortical_area_id: String,
    /// Number of distinct classes (= channels, one channel per class for classification).
    pub class_count: u32,
    /// Cortical-column depth (number of bins) of the motor area, used to decode each
    /// channel's activation. Must match the pinned genome's OPU area depth.
    pub bins: u32,
}
