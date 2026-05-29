//! The core crate for FEAGI. Defines the most common data structures used throughout

pub use feagi_data; // Expose feagi-data crate

mod templates;
mod feagi_json;
mod feagi_common_error;
pub mod genomic;
pub mod wgpu_temp;
pub mod useful_structs;
pub mod neuron_voxels;
pub mod neuron_descriptors;

pub use feagi_common_error::FeagiCommonError;
pub use feagi_json::FeagiJSON; // TODO delete me