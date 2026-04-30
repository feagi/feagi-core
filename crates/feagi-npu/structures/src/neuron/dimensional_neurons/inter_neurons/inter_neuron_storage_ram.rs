use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::inter_neurons::inter_neuron_traits::{InterNeuronCommonStorageTrait, InterNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::neuron_collection::NeuronCollectionRam;
use crate::neuron::dimensional_neurons::neuron_models::DimensionalNeuronModelDataResizableTrait;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
use crate::quantizables::NPUGlobalQuantization;

pub struct InterNeuronAllocRAMStorage<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
{
    neuron_collection: NeuronCollectionRam<Q, DNQ, NeuronModel>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> BaseNeuronCommonStorageTrait<Q, DNQ, NeuronModel> for InterNeuronAllocRAMStorage<Q, DNQ, NeuronModel> {
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<Q::CorticalIndexQuant> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> BaseNeuronResizableStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ, NeuronModel> {
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<DNQ::NeuronIndexQuant>) -> NeuronCount<DNQ::NeuronIndexQuant> {
        todo!()
    }

    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<Range<NPUNeuronIndex<DNQ::NeuronIndexQuant>>, FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> DimensionalNeuronCommonStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ, NeuronModel> {
    // TODO return bound data type
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> DimensionalNeuronResizableStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ, NeuronModel> {

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> InterNeuronCommonStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ, NeuronModel>
{
    // Custom Cortical Area Stuff
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> InterNeuronResizableStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ, NeuronModel>
{
    // TODO add cortical area (use spawners)
}