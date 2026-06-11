use feagi_structures::feagi_data::feagi_pdi::PDIExecutor;
use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagGenericDevice;
use crate::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantization;




pub trait NPUExecutorBurstCondenseFCLToFCLCompact<FGQ: FeagiStandardModelQuantization>:
PDIExecutor
+ PDITagGenericDevice
{

}



//region CPU Implementation

pub struct NPUExecutorBurstCondenseFCLToFCLCompactCPU<FGQ: FeagiStandardModelQuantization>
{
    _p: core::marker::PhantomData<FGQ>,
}

impl<FGQ: FeagiStandardModelQuantization> NPUExecutorBurstCondenseFCLToFCLCompactCPU<FGQ>
{
    pub fn condense_fcl(fcl_by_quants_table: X, fcl_compact_global_table: Y) {
        todo!()
    }
}
//endregion