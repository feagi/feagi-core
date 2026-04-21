// Top level error enum for this crate, holds errors from individual models

use crate::FeagiStructuresGenomicError;
use crate::neuron_voxels::FeagiStructuresNeuronVoxelError;
use crate::neurons::FeagiStructuresNeuronError;

// Note: The dynamic-String variants below are gated on the `alloc` feature so that the
// enum remains usable on bare no_std targets where only &'static str is available.
// The static-str variants (JSONError, InvalidValue) work in all configurations.
#[derive(Debug)]
pub enum FeagiStructuresError {
    NeuronVoxelError { neuron_voxel_error: FeagiStructuresNeuronVoxelError },
    NeuronError { neuron_error: FeagiStructuresNeuronError },
    GenomicError { genomic_error: FeagiStructuresGenomicError },
    JSONError { context: &'static str },
    InvalidValue { context: &'static str },

    #[cfg(feature = "alloc")]
    DeserializationError(String),
    #[cfg(feature = "alloc")]
    SerializationError(String),
    #[cfg(feature = "alloc")]
    BadParameters(String),
    #[cfg(feature = "alloc")]
    InternalError(String),
    /// Surface area that is deliberately not yet implemented. Used by higher-
    /// level crates (e.g. feagi-sensorimotor pipeline stages) to signal
    /// unsupported input/output paths without panicking.
    #[cfg(feature = "alloc")]
    NotImplemented(String),
}

#[cfg(feature = "std")]
impl core::fmt::Display for FeagiStructuresError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FeagiStructuresError::NeuronVoxelError { neuron_voxel_error } => {
                write!(f, "NeuronVoxelError: {:?}", neuron_voxel_error)
            }
            FeagiStructuresError::NeuronError { neuron_error } => {
                write!(f, "NeuronError: {:?}", neuron_error)
            }
            FeagiStructuresError::GenomicError { genomic_error } => {
                write!(f, "GenomicError: {:?}", genomic_error)
            }
            FeagiStructuresError::JSONError { context } => {
                write!(f, "JSONError: {}", context)
            }
            FeagiStructuresError::InvalidValue { context } => {
                write!(f, "InvalidValue: {}", context)
            }
            FeagiStructuresError::DeserializationError(msg) => {
                write!(f, "Failed to Deserialize: {}", msg)
            }
            FeagiStructuresError::SerializationError(msg) => {
                write!(f, "Failed to Serialize: {}", msg)
            }
            FeagiStructuresError::BadParameters(msg) => {
                write!(f, "Bad Parameters: {}", msg)
            }
            FeagiStructuresError::InternalError(msg) => {
                write!(f, "Internal Error (please report): {}", msg)
            }
            FeagiStructuresError::NotImplemented(msg) => {
                write!(f, "Not Implemented: {}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FeagiStructuresError {}

// TODO automatic From<> impls from sub-errors