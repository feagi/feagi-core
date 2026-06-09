use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::wrapped_indexing::{NPUNeuronIndexQuantizationLocal, NPUneuronFCLInputPotential};



pub trait QuantTypedFCLInputPotentialsCollection<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}

//region CPU implementation

pub struct QuantTypedFCLInputPotentialCollectionCPU<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>
{
    pub input_potentials: Vec<NPUneuronFCLInputPotential<NMQ::NeuronPotentialQuant>>,
}

impl <FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> QuantTypedFCLInputPotentialCollectionCPU<FGQ, NMQ>
{

}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDICollection for QuantTypedFCLInputPotentialCollectionCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDITagGenericDevice for QuantTypedFCLInputPotentialCollectionCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDITagCPU for QuantTypedFCLInputPotentialCollectionCPU<FGQ, NMQ> {}

impl <FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> QuantTypedFCLInputPotentialsCollection<FGQ, NMQ> for QuantTypedFCLInputPotentialCollectionCPU<FGQ, NMQ>
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant> {
        NPUNeuronIndexQuantizationLocal::wrap(&self.input_potentials.len())
    }
}



