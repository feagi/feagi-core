// TODO some things probably dont need to be exposed.

mod feagi_npu_structure_error;

pub mod quantizables;
pub mod neuron;
pub mod synapse;
pub mod fire_candidate_list;
pub mod executors;
pub mod connectome;


pub use feagi_npu_structure_error::FeagiNPUStructureError;