use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::wrapped_indexing::{NPUNeuronIndexQuantizationLocal, NPUNeuronMembranePotential};



pub trait TypedMembranePotentialsCollection<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}

//region CPU implementation

pub struct TypedMembranePotentialCollectionCPU<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>
{
    pub potentials: Vec<NPUNeuronMembranePotential<NMQ::NeuronPotentialQuant>>,
}

impl <FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> TypedMembranePotentialCollectionCPU<FGQ, NMQ>
{

}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDICollection for TypedMembranePotentialCollectionCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDITagGenericDevice for TypedMembranePotentialCollectionCPU<FGQ, NMQ> {}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> PDITagCPU for TypedMembranePotentialCollectionCPU<FGQ, NMQ> {}

impl <FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> TypedMembranePotentialsCollection<FGQ, NMQ> for TypedMembranePotentialCollectionCPU<FGQ, NMQ>
{
    fn get_number_contained_potentials_of_this_quantization(&self) -> NPUNeuronIndexQuantizationLocal<FGQ::NeuronIndexCountQuant> {
        NPUNeuronIndexQuantizationLocal::wrap(&self.potentials.len())
    }
}



