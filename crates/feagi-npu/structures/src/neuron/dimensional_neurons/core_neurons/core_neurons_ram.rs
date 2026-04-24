use core::ops::Range;
use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neurons::descriptors::{NeuronCount};
use feagi_structures::useful_structs::{IndexedDataTracker, RangeUintVector};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::core_neurons::default_core_areas::{CoreNeuronDeathDefaults, CoreNeuronFatigueDefaults, CoreNeuronPowerDefaults, NUMBER_SINGLE_NEURON_CORE_AREAS};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::shared_funcs_ram::{
    get_cortical_area_ref,
};
use crate::quantizables::{NPUQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential};


// NOTE: As of now, core is statically sized and we shall treat it as such

pub struct CoreNeuronAllocRAMStorage<Q: NPUQuantization>
{
    // Per Neuron (including invalids)
    neuron_global_burst_index_of_last_firing: [BurstGlobalIndex<Q::BurstIndexQuant>; NUMBER_SINGLE_NEURON_CORE_AREAS],
    neuron_membrane_potential: [NPUNeuronMembranePotential<Q::ValueQuant>; NUMBER_SINGLE_NEURON_CORE_AREAS],
    neuron_fire_threshold: [FireThreshold<Q::ValueQuant>; NUMBER_SINGLE_NEURON_CORE_AREAS],
    neuron_leak_coefficient: [LeakCoefficient<Q::PercentageQuant>; NUMBER_SINGLE_NEURON_CORE_AREAS],
    neuron_flags: [NeuronFlag; NUMBER_SINGLE_NEURON_CORE_AREAS],
    neuron_refractory_countdown: [BurstDelta<Q::BurstDeltaQuant>; NUMBER_SINGLE_NEURON_CORE_AREAS],
    neuron_consecutive_fire_count: [BurstDelta<Q::BurstDeltaQuant>; NUMBER_SINGLE_NEURON_CORE_AREAS], // how many times the neuron fired burst recently

    // Per Cortical Area (including invalids)
    cortical_datas: [DimensionalNeuronCorticalData<Q>; NUMBER_SINGLE_NEURON_CORE_AREAS],
    

}

// NOTE: Only define the constructor here, as we will be going through traits / generics for all data transfer!
impl<Q: NPUQuantization>
CoreNeuronAllocRAMStorage<Q>
{
    pub fn new() -> Self {
        Self {
            neuron_global_burst_index_of_last_firing:  [BurstGlobalIndex::ZERO; NUMBER_SINGLE_NEURON_CORE_AREAS],
            neuron_membrane_potential: [NPUNeuronMembranePotential::ZERO; NUMBER_SINGLE_NEURON_CORE_AREAS],
            neuron_fire_threshold: [
                CoreNeuronPowerDefaults::<Q>::DEFAULT_NEURON_FIRE_THRESHOLD,
                CoreNeuronDeathDefaults::<Q>::DEFAULT_NEURON_FIRE_THRESHOLD,
                CoreNeuronFatigueDefaults::<Q>::DEFAULT_NEURON_FIRE_THRESHOLD],
            neuron_leak_coefficient: [
                CoreNeuronPowerDefaults::<Q>::DEFAULT_NEURON_LEAK_COEFFICIENT,
                CoreNeuronDeathDefaults::<Q>::DEFAULT_NEURON_LEAK_COEFFICIENT,
                CoreNeuronFatigueDefaults::<Q>::DEFAULT_NEURON_LEAK_COEFFICIENT],
            neuron_flags: [
                CoreNeuronPowerDefaults::<Q>::DEFAULT_NEURON_FLAG,
                CoreNeuronDeathDefaults::<Q>::DEFAULT_NEURON_FLAG,
                CoreNeuronFatigueDefaults::<Q>::DEFAULT_NEURON_FLAG],
            neuron_refractory_countdown: [
                CoreNeuronPowerDefaults::<Q>::DEFAULT_NEURON_REFRACTORY_COUNTDOWN,
                CoreNeuronDeathDefaults::<Q>::DEFAULT_NEURON_REFRACTORY_COUNTDOWN,
                CoreNeuronFatigueDefaults::<Q>::DEFAULT_NEURON_REFRACTORY_COUNTDOWN],
            neuron_consecutive_fire_count: [
                CoreNeuronPowerDefaults::<Q>::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT,
                CoreNeuronDeathDefaults::<Q>::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT,
                CoreNeuronFatigueDefaults::<Q>::DEFAULT_NEURON_CONSECUTIVE_FIRE_COUNT],

            cortical_datas: [
                CoreNeuronPowerDefaults::<Q>::get_default_cortical_data(),
                CoreNeuronDeathDefaults::<Q>::get_default_cortical_data(),
                CoreNeuronFatigueDefaults::<Q>::get_default_cortical_data()],
        }
    }
    
    pub const NEURON_CORTICAL_AREA_INDEX: [CorticalAreaIndex<Q::CorticalIndexQuant>; 3] = [
        CorticalAreaIndex::DEFAULT_CORE_POWER,
        CorticalAreaIndex::DEFAULT_CORE_DEATH,
        CorticalAreaIndex::DEFAULT_CORE_FATIGUE
    ];

    fn get_cortical_area_ref(&self,  cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError> {
        if cortical_area_index > self.get_max_possible_cortical_area_index() {
            return Err(FeagiNPUNeuronError::InvalidCorticalIndex { context: "Only 3 Core Cortical areas exist! Given index out of range!", given_cortical_index: cortical_area_index.to_usize() as u32 })
        }
        Ok(&self.cortical_datas.get(cortical_area_index.to_usize()).unwrap())
    }

    fn get_cortical_area_ref_mut(&mut self,  cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&mut DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError> {
        if cortical_area_index > self.get_max_possible_cortical_area_index() {
            return Err(FeagiNPUNeuronError::InvalidCorticalIndex { context: "Only 3 Core Cortical areas exist! Given index out of range!", given_cortical_index: cortical_area_index.to_usize() as u32 })
        }
        Ok(self.cortical_datas.get_mut(cortical_area_index.to_usize()).unwrap())
    }

}


impl<Q: NPUQuantization>
DimensionalNeuronStaticStorageTrait<Q>
for CoreNeuronAllocRAMStorage<Q>
{
    fn get_cortical_data(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError> {
        if cortical_area_index > self.get_max_possible_cortical_area_index() {
            return Err(FeagiNPUNeuronError::InvalidCorticalIndex { context: "Only 3 Core Cortical areas exist! Given index out of range!", given_cortical_index: cortical_area_index.to_usize() as u32 })
        }
        Ok(&self.cortical_datas.get(cortical_area_index.to_usize()).unwrap())
    }

    fn get_global_burst_index_of_last_firing(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstGlobalIndex<Q::BurstIndexQuant>], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_global_burst_index_of_last_firing[range])
    }

    fn get_neuron_membrane_potential(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[NPUNeuronMembranePotential<Q::ValueQuant>], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_membrane_potential[range])
    }

    fn get_fire_threshold(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[FireThreshold<Q::ValueQuant>], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_fire_threshold[range])
    }

    fn get_leak_coefficient(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[LeakCoefficient<Q::PercentageQuant>], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_leak_coefficient[range])
    }

    fn get_neuron_flags(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[NeuronFlag], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_flags[range])
    }

    fn get_refractory_countdown(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstDelta<Q::BurstDeltaQuant>], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_refractory_countdown[range])
    }

    fn get_consecutive_fire_count(&self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<&[BurstDelta<Q::BurstDeltaQuant>], FeagiNPUNeuronError> {
        let range = &self.get_cortical_area_ref(cortical_area_index)?.neuron_range;
        let range = NPUNeuronIndex::to_usize_range(range.clone());
        Ok(&self.neuron_consecutive_fire_count[range])
    }

    /// Used to pass around slices easily at low cost for all cortical areas
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, Q> {
        DimensionalNeuronDataRefSliceAllCorticalAreas {
            neuron_cortical_area_index: &Self::NEURON_CORTICAL_AREA_INDEX,
            neuron_global_burst_index_of_last_firing: &mut self.neuron_global_burst_index_of_last_firing,
            neuron_membrane_potential: &mut self.neuron_membrane_potential,
            neuron_fire_threshold: &mut self.neuron_fire_threshold,
            neuron_leak_coefficient: &mut self.neuron_leak_coefficient,
            neuron_flags: &mut self.neuron_flags,
            neuron_refractory_countdown: &mut self.neuron_refractory_countdown,
            neuron_consecutive_fire_count: &mut self.neuron_consecutive_fire_count,

            cortical_data: &self.cortical_datas,
        }
    }

    /// Returns a struct of references to the slices of neuron data of a cortical index if it exists
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError> {
        let cortical_data = get_cortical_area_ref(&cortical_area_index, &Self::NEURON_CORTICAL_AREA_INDEX)?;
        let neuron_range = cortical_data.neuron_range.clone();
        let usize_range: Range<usize> = NPUNeuronIndex::<Q::NeuronIndexQuant>::to_usize_range(neuron_range.clone());

        Ok(DimensionalNeuronDataRefSliceSingleCorticalArea {
            neuron_cortical_area_index: &Self::NEURON_CORTICAL_AREA_INDEX[usize_range.clone()],
            neuron_global_burst_index_of_last_firing: &mut self.neuron_global_burst_index_of_last_firing[usize_range.clone()],
            neuron_membrane_potential: &mut self.neuron_membrane_potential[usize_range.clone()],
            neuron_fire_threshold: &mut self.neuron_fire_threshold[usize_range.clone()],
            neuron_leak_coefficient: &mut self.neuron_leak_coefficient[usize_range.clone()],
            neuron_flags: &mut self.neuron_flags[usize_range.clone()],
            neuron_refractory_countdown: &mut self.neuron_refractory_countdown[usize_range.clone()],
            neuron_consecutive_fire_count: &mut self.neuron_consecutive_fire_count[usize_range],

            cortical_data,
            global_neuron_index_range: neuron_range
        })
    }

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: CorticalAreaIndex<Q::CorticalIndexQuant>, executor: &impl NeuronFireThresholdExecutor<Q::ValueQuant, Q::CoordQuantQuant>) -> Result<(), FeagiNPUNeuronError> {
        let (usize_range, dimensions, neurons_per_voxel) = {
            let cortical_data = get_cortical_area_ref(&cortical_area_index, &self.cortical_datas)?;
            (
                NPUNeuronIndex::<Q::NeuronIndexQuant>::to_usize_range(cortical_data.neuron_range.clone()),
                cortical_data.dimensions,
                cortical_data.number_neurons_per_voxel,
            )
        };
        executor.set_new_fire_thresholds(
            &mut self.neuron_fire_threshold[usize_range.clone()],
            &self.neuron_flags[usize_range],
            &dimensions,
            neurons_per_voxel,
        )
    }
}


impl<Q: NPUQuantization>
DimensionalStaticStorageTrait<Q>
for CoreNeuronAllocRAMStorage<Q>
{

}


impl<Q: NPUQuantization>
BaseNeuronStaticStorageTrait<Q>
for CoreNeuronAllocRAMStorage<Q>
{
    const NUMBER_BYTES_PER_NEURON: usize = 0; // TODO

    /// Gets the maximum possible neuron index achievable by current quantization (or in the case
    /// of static implementations, the size of the neuron array).
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<Q::NeuronIndexQuant> {
        NPUNeuronIndex::MAX_VALUE
    }

    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<Q::NeuronIndexQuant> {
        NeuronCount::from_usize(NUMBER_SINGLE_NEURON_CORE_AREAS)
    }


    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<Q::NeuronIndexQuant> {
        NeuronCount::ZERO
    }


    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<Q::CorticalIndexQuant> {
        CorticalAreaIndex::from_usize(NUMBER_SINGLE_NEURON_CORE_AREAS) - CorticalAreaIndex::ONE
    }
}
