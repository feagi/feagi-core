
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalAreaCount, CorticalAreaIndex};
use feagi_structures::neurons::descriptors::{NeuronCount};
use crate::NPUCorticalAreaIdentifierFlag;
use crate::neuron::FeagiNPUNeuronError;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization, NPUNeuronIndex};
use crate::neuron::npu_storage::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::inter_neurons::inter_neuron_traits::{InterNeuronCommonStorageTrait, InterNeuronResizableStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::cortical_area_collection::ResizableCorticalAreaCollectionRam;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::DimensionalNeuronModelDataResizableTrait;

pub struct InterNeuronStorageResizableRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
{
    neuron_collection: ResizableCorticalAreaCollectionRam<Q, DNQ, NeuronModel>,
    total_number_live_neurons: NeuronCount<Q::NeuronIndexCountQuant>,
    total_number_dead_neurons: NeuronCount<Q::NeuronIndexCountQuant>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
BaseNeuronCommonStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    const TYPE_CORTICAL_AREA: NPUCorticalAreaIdentifierFlag =
        NPUCorticalAreaIdentifierFlag::from_quantization_and_model(
            DNQ::GENERAL_CORTICAL_AREA_QUANTIZATION_LEVEL,
            NeuronModel::CORTICAL_AREA_MODEL_TYPE);

    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<Q::NeuronIndexCountQuant> {
        NPUNeuronIndex::MAX_VALUE
    }

    fn get_total_number_of_live_neurons(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
        self.total_number_live_neurons
    }

    fn get_total_number_of_dead_neurons(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
        self.total_number_dead_neurons
    }

    fn get_number_live_cortical_areas(&self) -> CorticalAreaCount<Q::CorticalIndexCountQuant>
    {
        self.neuron_collection.get_number_live_cortical_areas()
    }

    fn get_number_dead_cortical_areas(&self) -> CorticalAreaCount<Q::CorticalIndexCountQuant>
    {
        self.neuron_collection.get_number_dead_cortical_areas()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
BaseNeuronResizableStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<Q::NeuronIndexCountQuant>) -> NeuronCount<Q::NeuronIndexCountQuant> {
        todo!()
    }

    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<(), FeagiNPUNeuronError> {
        self.neuron_collection.mark_cortical_area_as_dead(cortical_index)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
DimensionalNeuronCommonStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    type DimensionalNeuronModelDataType = NeuronModel;

    fn get_cortical_area_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        self.neuron_collection.get_cortical_area(cortical_area_index)
    }

    fn get_cortical_area_data_mut(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&mut Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        self.neuron_collection.get_cortical_area_mut(cortical_area_index)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
DimensionalNeuronResizableStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
InterNeuronCommonStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    type NeuronModelType = NeuronModel;
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
InterNeuronResizableStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    fn add_interneuron_cortical_area(&mut self, cortical_area_generator: &impl DimensionalCorticalAreaGeneratorTrait<Q, DNQ>)
        -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUNeuronError> {
        self.neuron_collection.add_cortical_area(cortical_area_generator)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    pub fn new() -> Self {
        Self {
            neuron_collection: ResizableCorticalAreaCollectionRam::new(),
            total_number_live_neurons: NeuronCount::ZERO,
            total_number_dead_neurons: NeuronCount::ZERO,
        }
    }
}