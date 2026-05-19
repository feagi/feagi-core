// TODO some things probably dont need to be exposed.





pub mod quantizables;
pub mod neuron;
pub mod synapse;
pub mod fire_candidate_list;
pub mod fire_queue;
pub mod executors;
pub mod connectome;

mod burst_engines;
mod cortical_area_identifier_flag;
mod typed_indexing;
mod feagi_npu_structure_error;
mod dynamics;

pub use feagi_npu_structure_error::FeagiNPUStructureError;
pub use cortical_area_identifier_flag::NPUCorticalAreaIdentifierFlag;
pub use typed_indexing::{CorticalTypedNeuronIndex, CorticalTypedCorticalIndex};