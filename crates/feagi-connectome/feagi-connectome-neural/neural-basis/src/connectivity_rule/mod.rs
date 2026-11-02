pub mod synapse_mapping;

use feagi_basis::create_wrapped_quantized_unsigned_integer;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;













//region Kernel Type


pub trait ConnectivityRuleSynapseKernel<FIQ: FeagiIndexQuantization> {

}

create_wrapped_quantized_unsigned_integer!(
    /// Index of a synapse relative to its Axon Bundle
    pub AxonBundleSynapseLocalIndex
);



//endregion