use core::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_neuron::{NPUNeuronIndexQuantizationLocal, NPUNeuronMembranePotential};

pub trait CPQTypedFCLCollection<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}

//region CPU implementation

pub struct CPQTypedFCLCollectionCPU<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization>
{
    pub input_potentials: Vec<NPUNeuronMembranePotential<CPQ::NeuronPotentialQuant>>,
    _p: PhantomData<FGQ>,
}

impl <FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> CPQTypedFCLCollectionCPU<FGQ, CPQ>
{

}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDICollection for CPQTypedFCLCollectionCPU<FGQ, CPQ> {}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDITagGenericDevice for CPQTypedFCLCollectionCPU<FGQ, CPQ> {}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDITagCPU for CPQTypedFCLCollectionCPU<FGQ, CPQ> {}

impl <FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> CPQTypedFCLCollection<FGQ, CPQ> for CPQTypedFCLCollectionCPU<FGQ, CPQ>
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant> {
        NPUNeuronIndexQuantizationLocal::wrap(&self.input_potentials.len())
    }
}



