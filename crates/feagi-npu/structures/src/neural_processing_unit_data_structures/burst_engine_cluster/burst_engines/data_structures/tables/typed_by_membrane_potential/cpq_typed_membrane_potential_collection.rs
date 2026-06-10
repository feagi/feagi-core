use core::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, CorticalPotentialQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_neuron::{NPUNeuronIndexQuantizationLocal, NPUNeuronMembranePotential};

pub trait CPQTypedMembranePotentialsCollection<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}

//region CPU implementation

pub struct CPQTypedMembranePotentialCollectionCPU<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization>
{
    pub potentials: Vec<NPUNeuronMembranePotential<CPQ::NeuronPotentialQuant>>,
    _p: PhantomData<FGQ>,
}

impl <FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> CPQTypedMembranePotentialCollectionCPU<FGQ, CPQ>
{

}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDICollection for CPQTypedMembranePotentialCollectionCPU<FGQ, CPQ> {}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDITagGenericDevice for CPQTypedMembranePotentialCollectionCPU<FGQ, CPQ> {}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDITagCPU for CPQTypedMembranePotentialCollectionCPU<FGQ, CPQ> {}

impl <FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> CPQTypedMembranePotentialsCollection<FGQ, CPQ> for CPQTypedMembranePotentialCollectionCPU<FGQ, CPQ>
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant> {
        NPUNeuronIndexQuantizationLocal::wrap(&self.potentials.len())
    }
}



