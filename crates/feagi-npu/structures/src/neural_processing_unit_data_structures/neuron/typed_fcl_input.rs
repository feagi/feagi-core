use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::wrapped_indexing::{NPUNeuronIndexQuantizationLocal, NPUneuronFCLInputPotential};



pub trait TypedFCLInputPotentialsCollection<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}

//region CPU implementation

pub struct TypedFCLInputPotentialCollectionCPU<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>
{
    pub input_potentials: Vec<NPUneuronFCLInputPotential<NMQ::NeuronPotentialQuant>>,
}

impl <FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> TypedFCLInputPotentialCollectionCPU<FGQ, NMQ>
{

}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDICollection for TypedFCLInputPotentialCollectionCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDITagGenericDevice for TypedFCLInputPotentialCollectionCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDITagCPU for TypedFCLInputPotentialCollectionCPU<FGQ, NMQ> {}

impl <FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> TypedFCLInputPotentialsCollection<FGQ, NMQ> for TypedFCLInputPotentialCollectionCPU<FGQ, NMQ>
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant> {
        NPUNeuronIndexQuantizationLocal::wrap(&self.input_potentials.len())
    }
}



