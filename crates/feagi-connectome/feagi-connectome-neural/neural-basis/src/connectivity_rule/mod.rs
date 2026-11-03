pub mod dimensional;

use feagi_basis::create_wrapped_quantized_unsigned_integer;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;




pub trait ConnectivityRule<FIQ: FeagiIndexQuantization> {
    type MappingInputLayout;
    type MappingOutputLayout;
}








//region Kernel Type


pub trait ConnectivityRuleSynapseKernel<FIQ: FeagiIndexQuantization> {

}

create_wrapped_quantized_unsigned_integer!(
    /// Index of a synapse relative to its Axon Bundle
    pub AxonBundleSynapseLocalIndex
);



//endregion