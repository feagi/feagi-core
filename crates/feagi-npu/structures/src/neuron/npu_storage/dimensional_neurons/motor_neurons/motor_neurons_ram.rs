use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalAreaCount, CorticalAreaIndex};
use feagi_structures::neurons::descriptors::{NeuronCount};
use crate::neuron::npu_storage::base_storage_traits::{BaseNeuronResizableStorageTrait, BaseNeuronCommonStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::FeagiStandardCorticalAreaGenerator;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::ram::structs_ram::FeagiStandardNeuronDataRam;
use crate::neuron::npu_storage::dimensional_neurons::cortical_area_collection::ResizableCorticalAreaCollectionRam;
use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronResizableStorageTrait, DimensionalNeuronCommonStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::motor_neurons::motor_neuron_traits::{MotorNeuronCommonStorageTrait, MotorNeuronResizableStorageTrait};
use crate::NPUCorticalAreaIdentifierFlag;
use crate::quantizables::{NPUGlobalQuantization, NPUNeuronIndex,NPUDimensionalNeuronQuantization};
// In this implementation, we can do a lot by keeping neurons of a cortical area grouped together, albeit they may not be guaranteed to be in cortical index order

// TODO motor traits
// TODO just copying inter neurons for now, but we should have some sensiomotor / motor specific implementations
pub struct MotorNeuronStorageResizableRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    neuron_collection: ResizableCorticalAreaCollectionRam<Q, DNQ, FeagiStandardNeuronDataRam<Q, DNQ>>,
    total_number_live_neurons: NeuronCount<DNQ::NeuronIndexCountQuant>,
    total_number_dead_neurons: NeuronCount<DNQ::NeuronIndexCountQuant>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronCommonStorageTrait<Q, DNQ> for MotorNeuronStorageResizableRam<Q, DNQ> {
    const TYPE_CORTICAL_AREA: NPUCorticalAreaIdentifierFlag =
        NPUCorticalAreaIdentifierFlag::MotorFeagiStandard(DNQ::GENERAL_DATA_QUANTIZATION_LEVEL);

    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<DNQ::NeuronIndexCountQuant> {
        NPUNeuronIndex::MAX_VALUE
    }

    fn get_total_number_of_live_neurons(&self) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        self.total_number_live_neurons
    }

    fn get_total_number_of_dead_neurons(&self) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
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

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronResizableStorageTrait<Q, DNQ> for MotorNeuronStorageResizableRam<Q, DNQ> {
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<DNQ::NeuronIndexCountQuant>) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        todo!()
    }

    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<(), FeagiNPUNeuronError> {
        self.neuron_collection.mark_cortical_area_as_dead(cortical_index)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronCommonStorageTrait<Q, DNQ> for MotorNeuronStorageResizableRam<Q, DNQ> {
    type DimensionalNeuronModelDataType = FeagiStandardNeuronDataRam<Q, DNQ>;

    fn get_cortical_area_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        self.neuron_collection.get_cortical_area(cortical_area_index)
    }

    fn get_cortical_area_data_mut(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&mut Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        self.neuron_collection.get_cortical_area_mut(cortical_area_index)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronResizableStorageTrait<Q, DNQ> for MotorNeuronStorageResizableRam<Q, DNQ> {}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> MotorNeuronCommonStorageTrait<Q, DNQ> for MotorNeuronStorageResizableRam<Q, DNQ> {}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
MotorNeuronResizableStorageTrait<Q, DNQ> for MotorNeuronStorageResizableRam<Q, DNQ> {
    fn add_motor_cortical_area(&mut self, cortical_area_generator: &impl FeagiStandardCorticalAreaGenerator<Q, DNQ>) -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUNeuronError> {
        self.neuron_collection.add_cortical_area(cortical_area_generator)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> MotorNeuronStorageResizableRam<Q, DNQ> {
    pub fn new() -> Self {
        Self {
            neuron_collection: ResizableCorticalAreaCollectionRam::new(),
            total_number_live_neurons: NeuronCount::ZERO,
            total_number_dead_neurons: NeuronCount::ZERO,
        }
    }
}

















