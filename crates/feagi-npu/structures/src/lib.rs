// TODO some things probably dont need to be exposed.

mod feagi_npu_data_error;

pub mod descriptors;
pub mod neuron;
pub mod synapse;
pub mod fire_candidate_list;
pub mod executors;
pub mod connectome;


pub use feagi_npu_data_error::FeagiNPUDataError;