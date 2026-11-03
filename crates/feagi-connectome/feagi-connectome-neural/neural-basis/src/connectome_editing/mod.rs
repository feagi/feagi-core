use crate::neuron_model::interfacing::generators::CorticalAreaGeneratorDispatch;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub mod connectome_edit_steps;
pub mod error;

pub trait ConnectomeVisitorDispatchWrapper<FIQ: FeagiIndexQuantization> {
    type CorticalAreaGeneratorDispatch: CorticalAreaGeneratorDispatch<FIQ>;
}

