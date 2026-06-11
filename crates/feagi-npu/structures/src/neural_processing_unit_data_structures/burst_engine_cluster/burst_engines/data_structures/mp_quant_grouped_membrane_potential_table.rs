use core::marker::PhantomData;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantization};
use feagi_structures::feagi_data::SupportsUintOps;
use crate::neural_processing_unit_data_structures::cpu_wrappers::{NPUWrappedNeuronIndexF32Quant, NPUWrappedNeuronMembranePotential};

/// Holds membrane potential data, in subgroups by quantization, of all neurons in the
/// connectome. Globally, is ordered from f32, f8, f16, f64, u8
pub trait NPUNeuronPotentialsGroupedTable<FGQ: FeagiGlobalQuantization> {}


// TODO other quants!
type F32Quant = <CorticalPotentialQuantizationFloat32 as CorticalPotentialQuantization>::NeuronPotentialQuant;


pub struct NPUNeuronMembranePotentialsGroupedTableCPU<FGQ: FeagiGlobalQuantization>
{
    pub float_32: Vec<NPUWrappedNeuronMembranePotential<F32Quant>>,
    // TODO f8
    // TODO f16
    // TODO f64
    // TODO u8?

    _p: PhantomData<FGQ>,

}

impl<FGQ: FeagiGlobalQuantization> NPUNeuronMembranePotentialsGroupedTableCPU<FGQ> {

    // TODO other quants

    #[inline(always)]
    pub fn get_float_32(&self, index: NPUWrappedNeuronIndexF32Quant<FGQ::NeuronIndexCountQuant>)
        -> &NPUWrappedNeuronMembranePotential<F32Quant>
    {
        &self.float_32[index.to_usize()]
    }

    #[inline(always)]
    pub fn get_float_32_mut(&mut self, index: NPUWrappedNeuronIndexF32Quant<FGQ::NeuronIndexCountQuant>)
                        -> &mut NPUWrappedNeuronMembranePotential<F32Quant>
    {
        &mut self.float_32[index.to_usize()]
    }

}












