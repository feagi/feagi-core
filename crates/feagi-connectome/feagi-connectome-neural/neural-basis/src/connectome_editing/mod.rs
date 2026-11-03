use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub mod connectome_edit_steps;
pub mod error;

pub trait ConnectomeVisitorDispatchWrapper<FIQ: FeagiIndexQuantization> {
    type CorticalAreaSpecDispatch: Send;
}

