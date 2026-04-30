// TODO some things probably dont need to be exposed.


mod feagi_npu_structure_error;

pub mod quantizables;
pub mod neuron;
pub mod synapse;
pub mod fire_candidate_list;
pub mod fire_queue;
pub mod executors;
pub mod connectome;
mod burst_engines;
mod cortical_area_identifier_flag;

pub use feagi_npu_structure_error::FeagiNPUStructureError;