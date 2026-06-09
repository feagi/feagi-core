use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_neuron::NPUNeuronIndexQuantizationLocal;

pub trait NPUInputFCLCollection<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}

//region CPU implementation

pub struct QuantTypedFCLInputPotentialCollectionCPU<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization>
{
    pub input_potentials: Vec<NPUneuronFCLInputPotential<CPQ::NeuronPotentialQuant>>,
}

impl <FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> QuantTypedFCLInputPotentialCollectionCPU<FGQ, CPQ>
{

}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDICollection for QuantTypedFCLInputPotentialCollectionCPU<FGQ, CPQ> {}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDITagGenericDevice for QuantTypedFCLInputPotentialCollectionCPU<FGQ, CPQ> {}

impl<FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> PDITagCPU for QuantTypedFCLInputPotentialCollectionCPU<FGQ, CPQ> {}

impl <FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization> NPUInputFCLCollection<FGQ, CPQ> for QuantTypedFCLInputPotentialCollectionCPU<FGQ, CPQ>
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant> {
        NPUNeuronIndexQuantizationLocal::wrap(&self.input_potentials.len())
    }
}



