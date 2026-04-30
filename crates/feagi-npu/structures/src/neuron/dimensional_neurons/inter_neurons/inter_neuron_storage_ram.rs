use core::ops::Range;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_storage_traits::{BaseNeuronCommonStorageTrait, BaseNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::dimensional_storage_traits::{DimensionalNeuronCommonStorageTrait, DimensionalNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::inter_neurons::inter_neuron_traits::{InterNeuronCommonStorageTrait, InterNeuronResizableStorageTrait};
use crate::neuron::dimensional_neurons::neuron_collection::ResizableCorticalAreaCollectionRam;
use crate::neuron::dimensional_neurons::neuron_models::DimensionalNeuronModelDataResizableTrait;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUDimensionalNeuronQuantization, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability};
use crate::quantizables::NPUGlobalQuantization;

pub struct InterNeuronAllocRAMStorage<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
{
    neuron_collection: ResizableCorticalAreaCollectionRam<Q, DNQ, NeuronModel>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronCommonStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ> {
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

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> BaseNeuronResizableStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ> {
    fn free_unused_neuron_capacity(&mut self, spare_capacity_to_maintain: NeuronCount<DNQ::NeuronIndexCountQuant>) -> NeuronCount<DNQ::NeuronIndexCountQuant> {
        todo!()
    }

    fn delete_cortical_area(&mut self, cortical_index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<Range<NPUNeuronIndex<DNQ::NeuronIndexCountQuant>>, FeagiNPUNeuronError> {
        todo!()
    }
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronCommonStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ> {
    // TODO return bound data type
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> DimensionalNeuronResizableStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ> {

}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronCommonStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ>
{
    // Custom Cortical Area Stuff
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> InterNeuronResizableStorageTrait<Q, DNQ> for InterNeuronAllocRAMStorage<Q, DNQ>
{
    // TODO add cortical area (use spawners)
}