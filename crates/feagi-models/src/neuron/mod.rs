pub mod genome_interface;
pub mod model_extensions;
pub mod models;
/// A neuron models base definition trait. Is extended with other functionality via other traits
pub mod neuron_model;
/// The actual data of a neuron model. All neuron models must have some cortical level data defined
/// and optionally (most likely) needs per neuron data defined as well
pub mod neuron_model_data;
pub mod neuron_model_quantization;


// TODO Use build.rs to automatically build enums and include! to bring them here!
pub mod model_and_quantization;
pub mod npu_requests;
pub mod cortical_area_layout;
