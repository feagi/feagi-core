use core::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neurons::descriptors::{NeuronCount};
use feagi_structures::useful_structs::{IndexedDataTracker, RangeUintVector};
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::neuron::base_dimension_traits::{DimensionalStaticStorageTrait};
use crate::neuron::base_traits::{BaseNeuronStaticStorageTrait};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataRefSliceAllCorticalAreas, DimensionalNeuronDataRefSliceSingleCorticalArea};
use crate::neuron::dimensional_neurons::dimensional_traits::{DimensionalNeuronStaticStorageTrait};
use crate::neuron::dimensional_neurons::shared_funcs_ram::{
    get_cortical_area_ref,
};
use crate::quantizables::{NPUQuantization, BurstDelta, BurstGlobalIndex, FireThreshold, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential};

// TODO Core Traits
// TODO right now we are copying every other interneuron. However, we know core areas are rather constant. we should model that with that in mind

pub struct CoreNeuronAllocRAMStorage<Q: NPUQuantization>
{
    // Per Neuron (including invalids)
    neuron_cortical_area_index: Vec<CorticalAreaIndex<Q::CorticalIndex>>, // faster than potentially reverse looking up a large hashmap
    neuron_global_burst_index_of_last_firing: Vec<BurstGlobalIndex<Q::BurstIndex>>,
    neuron_membrane_potential: Vec<NPUNeuronMembranePotential<Q::Value>>,
    neuron_fire_threshold: Vec<FireThreshold<Q::Value>>,
    neuron_leak_coefficient: Vec<LeakCoefficient<Q::Percentage>>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<BurstDelta<Q::BurstDelta>>,
    neuron_consecutive_fire_count: Vec<BurstDelta<Q::BurstDelta>>, // how many times the neuron fired burst recently

    // Per Cortical Area (including invalids)
    cortical_datas: IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>,

    // Cached Data
    cache_number_valid_neurons: NeuronCount<Q::NeuronIndex>,
    cache_number_invalid_neurons: NeuronCount<Q::NeuronIndex>,
    cache_invalid_neuron_index_blocks: RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>, NeuronCount<Q::NeuronIndex>>,
}

// NOTE: Only define the constructor here, as we will be going through traits / generics for all data transfer!
impl<Q: NPUQuantization>
CoreNeuronAllocRAMStorage<Q>
{
    pub fn new(number_neurons_to_preallocate_space_for: NeuronCount<Q::NeuronIndex>, number_cortical_areas_to_preallocate_space_for: CorticalAreaIndex<Q::CorticalIndex>) -> Self {
        Self {
            neuron_cortical_area_index: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_global_burst_index_of_last_firing: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_membrane_potential: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_fire_threshold: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_leak_coefficient: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_flags: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_refractory_countdown: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),
            neuron_consecutive_fire_count: Vec::with_capacity(number_neurons_to_preallocate_space_for.to_usize()),

            cortical_datas: IndexedDataTracker::with_capacity(number_cortical_areas_to_preallocate_space_for),

            cache_number_valid_neurons: NeuronCount::ZERO,
            cache_number_invalid_neurons: NeuronCount::ZERO,
            cache_invalid_neuron_index_blocks: RangeUintVector::new(),
        }
    }

    // TODO move to core specific functions, particularly for initialization

}


impl<Q: NPUQuantization>
DimensionalNeuronStaticStorageTrait<Q>
for CoreNeuronAllocRAMStorage<Q>
{

    /// Used to pass around slices easily at low cost for all cortical areas
    fn get_neuron_values_of_all_dimensional_neuron_cortical_areas_to_process(&mut self) -> DimensionalNeuronDataRefSliceAllCorticalAreas<'_, Q> {
        DimensionalNeuronDataRefSliceAllCorticalAreas {
            neuron_cortical_area_index: &self.neuron_cortical_area_index,
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
    fn get_neuron_values_of_specific_dimensional_neuron_cortical_area_to_process(&mut self, cortical_area_index: Q::CorticalIndex) -> Result<DimensionalNeuronDataRefSliceSingleCorticalArea<'_, Q>, FeagiNPUNeuronError> {
        let cortical_area_index = CorticalAreaIndex(cortical_area_index);
        let cortical_data = get_cortical_area_ref(&cortical_area_index, &self.cortical_datas)?;
        let neuron_range = cortical_data.neuron_range.clone();
        let usize_range: Range<usize> = NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(neuron_range.clone());

        Ok(DimensionalNeuronDataRefSliceSingleCorticalArea {
            neuron_cortical_area_index: &self.neuron_cortical_area_index[usize_range.clone()],
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

    fn set_neuron_fire_threshold(&mut self, cortical_area_index: Q::CorticalIndex, executor: &impl NeuronFireThresholdExecutor<Q::Value, Q::Coord>) -> Result<(), FeagiNPUNeuronError> {
        let cortical_area_index = CorticalAreaIndex(cortical_area_index);
        let (usize_range, dimensions, neurons_per_voxel) = {
            let cortical_data = get_cortical_area_ref(&cortical_area_index, &self.cortical_datas)?;
            (
                NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(cortical_data.neuron_range.clone()),
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
    fn get_max_possible_neuron_index(&self) -> NPUNeuronIndex<Q::NeuronIndex> {
        NPUNeuronIndex::MAX_VALUE
    }

    /// Returns the count of valid neurons in the structure. NOT THE SAME AS TOTAL NUMBER OF
    /// NEURONS STORED!
    fn get_total_number_of_valid_neurons(&self) -> NeuronCount<Q::NeuronIndex> {
        self.cache_number_valid_neurons
    }


    /// Returns the count of invalid neurons in the structure. NOT THE SAME AS TOTAL FREE CAPACITY!
    fn get_total_number_of_invalid_neurons(&self) -> NeuronCount<Q::NeuronIndex> {
        self.cache_number_invalid_neurons
    }


    fn get_max_possible_cortical_area_index(&self) -> CorticalAreaIndex<Q::CorticalIndex> {
        CorticalAreaIndex::MAX_VALUE
    }
}
