use ahash::HashMap;
use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::identifiers::cortical_id::CorticalID;

pub struct ComposableRunningCorticalLookup<FIQ: FeagiIndexQuantization>(HashMap<CorticalID, ()>); // TODO internal is index



impl<FIQ: FeagiIndexQuantization> ComposableRunningCorticalLookup<FIQ> {

}

pub struct ComposableRunningCorticalModelLookup<FIQ: FeagiIndexQuantization, NM> {

}

