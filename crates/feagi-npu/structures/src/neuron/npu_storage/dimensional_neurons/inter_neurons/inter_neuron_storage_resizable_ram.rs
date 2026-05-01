use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::npu_storage::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::inter_neurons::inter_neuron_traits::{InterNeuronCommonStorageTrait, InterNeuronResizableStorageTrait};
use crate::neuron::npu_storage::dimensional_neurons::cortical_area_collection::ResizableCorticalAreaCollectionRam;
use crate::neuron::neuron_models::DimensionalNeuronModelDataResizableTrait;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::NPUCorticalAreaIdentifierFlag;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
use crate::quantizables::NPUGlobalQuantization;
use crate::typed_indexing::{CorticalTypedCorticalIndex, CorticalTypedNeuronIndex};

pub struct InterNeuronStorageResizableRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
{
    neuron_collection: ResizableCorticalAreaCollectionRam<Q, DNQ, NeuronModel>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
BaseNeuronCommonStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    const TYPE_CORTICAL_AREA: NPUCorticalAreaIdentifierFlag = NeuronModel::NEURON_MODEL_TYPE;
    
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<DNQ::NeuronIndexCountQuant> {
        todo!()
    }

    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        todo!()
    }

    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        todo!()
    }

    fn get_number_cortical_areas(&self) -> CorticalAreaIndex<Q::CorticalIndexCountQuant> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
BaseNeuronResizableStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<DNQ::NeuronIndexCountQuant>) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        todo!()
    }

    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<Range<NPUNeuronIndex<DNQ::NeuronIndexCountQuant>>, FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
DimensionalNeuronCommonStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    type DimensionalNeuronModelDataType = FeagiStandardNeuronDataRam<Q, DNQ>;

    fn get_cortical_area_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        todo!()
    }

    fn get_cortical_area_data_mut(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&mut Self::DimensionalNeuronModelDataType, FeagiNPUNeuronError> {
        todo!()
    }
    // TODO return bound data type
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
DimensionalNeuronResizableStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    // TODO resize using some sort of trait resizer struct that takes a neuron model type
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
InterNeuronCommonStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    // Custom Cortical Area Stuff
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
InterNeuronResizableStorageTrait<Q, DNQ> for InterNeuronStorageResizableRam<Q, DNQ, NeuronModel>
{
    // TODO add cortical area (use spawners)
}