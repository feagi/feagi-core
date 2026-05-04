
use feagi_structures::base_quantizable::{QuantizableUIntType};
use feagi_structures::genomic::cortical_area::descriptors::{CorticalAreaCount, CorticalAreaIndex};
use feagi_structures::neurons::descriptors::{NeuronCount};
use crate::neuron::npu_storage::base_storage_traits::{BaseNeuronCommonStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::feagi_standard::ram::structs_ram::FeagiStandardNeuronDataRam;
use crate::neuron::npu_storage::dimensional_neurons::core_neurons::core_neuron_traits::CoreNeuronCommonStorageTrait;
use crate::neuron::npu_storage::dimensional_neurons::cortical_area_collection::ResizableCorticalAreaCollectionRam;
use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::DimensionalNeuronCommonStorageTrait;
use crate::NPUCorticalAreaIdentifierFlag;
use crate::quantizables::{NPUGlobalQuantization, NPUNeuronIndex, NPUDimensionalNeuronQuantization};



pub struct CoreNeuronStorageResizableRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization>
{
    neuron_collection: ResizableCorticalAreaCollectionRam<Q, DNQ, FeagiStandardNeuronDataRam<Q, DNQ>>, // TODO we dont need something resizable as we will never add cortical areas
    total_number_live_neurons: NeuronCount<Q::NeuronIndexCountQuant>,
    total_number_dead_neurons: NeuronCount<Q::NeuronIndexCountQuant>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronCommonStorageTrait<Q, DNQ> for CoreNeuronStorageResizableRam<Q, DNQ> {
    const TYPE_CORTICAL_AREA: NPUCorticalAreaIdentifierFlag =
        NPUCorticalAreaIdentifierFlag::CoreFeagiStandard(DNQ::GENERAL_CORTICAL_AREA_QUANTIZATION_LEVEL);

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

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronCommonStorageTrait<Q, DNQ> for CoreNeuronStorageResizableRam<Q, DNQ> {
    type DimensionalNeuronModelDataType = FeagiStandardNeuronDataRam<Q, DNQ>;

    fn get_cortical_area_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        self.neuron_collection.get_cortical_area(cortical_area_index)
    }

    fn get_cortical_area_data_mut(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&mut Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        self.neuron_collection.get_cortical_area_mut(cortical_area_index)
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> CoreNeuronCommonStorageTrait<Q, DNQ> for CoreNeuronStorageResizableRam<Q, DNQ> {}


impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> CoreNeuronStorageResizableRam<Q, DNQ> {
    pub fn new() -> Self {
        Self {
            neuron_collection: ResizableCorticalAreaCollectionRam::new(),
            total_number_live_neurons: NeuronCount::ZERO,
            total_number_dead_neurons: NeuronCount::ZERO,
        }
    }
    // TODO should new create the core areas initially?
}