//! The core crate for FEAGI. Defines the most common data structures used throughout

// Reexported so the wrapper-generating macros ($crate::...) resolve these
// sibling crates regardless of which crate invokes them.
pub use feagi_common_quantizable;
pub use feagi_common_spatial;


mod templates;
mod feagi_json;
mod feagi_common_error;
pub mod genomic;
pub mod useful_structs;
pub mod neuron_voxels;
pub mod neuron_descriptors;
pub mod data_wrappers;
pub mod quantization_levels;



pub use feagi_common_error::FeagiCommonError;
pub use feagi_json::FeagiJSON; // TODO delete me