use core::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::{IndexedDataTracker, RangeUintVector};
use crate::neuron::defaults::DimensionalNeuronDefaults;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalNeuronDataFromCorticalArea};
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::{DimensionalNeuronCorticalFlag, NeuronFlag};
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NPUNeuronMembranePotential, NeuronExcitability, NPUQuantization};

/// Get the cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
pub(crate) fn get_cortical_area_ref<'a, Q: NPUQuantization>(cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>
                                                            , cortical_data: &'a IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>)
    -> Result<&'a DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError>
{
    Ok(cortical_data.get(*cortical_area_index)
        .ok_or_else(|| FeagiNPUNeuronError::InvalidCorticalIndex{
            context: "Requested Cortical Area Index does not exist!",
            given_cortical_index: cortical_area_index.to_usize() as u32
        })?)
}

/// Get the mutable cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
pub(crate) fn get_cortical_area_ref_mut<'a, Q: NPUQuantization>(cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>, cortical_data: &'a mut IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>)
    -> Result<&'a mut DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError>
{
    Ok(cortical_data.get_mut(*cortical_area_index)
        .ok_or_else(|| FeagiNPUNeuronError::InvalidCorticalIndex{
            context: "Requested Cortical Area Index does not exist!",
            given_cortical_index: cortical_area_index.to_usize() as u32
        })?)
}


/// Marks the neurons of a cortical area as invalid, as well as other cache work in this regard.
/// Returns the range of neuron indexes invalidated.
pub(crate) fn invalidate_cortical_area_and_return_invalidated_neuron_range<Q: NPUQuantization>(
    cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>,
    cortical_data: &mut IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>,
    neuron_flags: &mut Vec<NeuronFlag>,
    number_valid_neurons: &mut NeuronCount<Q::NeuronIndex>,
    number_invalid_neurons: &mut NeuronCount<Q::NeuronIndex>,
    invalid_neuron_index_blocks: &mut RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>, NeuronCount<Q::NeuronIndex>>)
    -> Result<Range<NPUNeuronIndex<Q::NeuronIndex>>, FeagiNPUNeuronError> {


    let cortical_data = get_cortical_area_ref_mut(&cortical_area_index, cortical_data)?;

    // so, since we actually do not care for any other flag in the neuron data except for
    // the is valid flag being set to false, just mass fill the area with the bitpack containing
    // that setting
    let neuron_range_usize: Range<usize> = NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(cortical_data.neuron_range.clone());
    let neuron_flag_slice: &mut [NeuronFlag] = &mut neuron_flags[neuron_range_usize];
    let invalid_flag = NeuronFlag::ALL_ZEROS;
    neuron_flag_slice.fill(invalid_flag);

    let number_of_neurons: NeuronCount<Q::NeuronIndex> = NPUNeuronIndex::get_count_from_block(&cortical_data.neuron_range);

    // Some neurons may have died on their own
    let number_of_neurons_invalidated = number_of_neurons - cortical_data.number_neurons_invalid_from_degeneration;


    // Mark neurons as dead in the cache too
    *number_valid_neurons -= number_of_neurons_invalidated;
    *number_invalid_neurons += number_of_neurons_invalidated;
    invalid_neuron_index_blocks.add_range(cortical_data.neuron_range.clone());

    Ok(cortical_data.neuron_range.clone())
}


/// Creates a cortical area in RAM-backed parallel neuron arrays, filling every neuron with the
/// given uniform values. Attempts to reuse a previously invalidated region if one is big enough,
/// otherwise appends to the end of the neuron arrays.
///
/// Returns the assigned cortical area index and the range of neuron indexes it covers.
pub(crate) fn default_create_cortical_area_with_uniform_neurons<Q: NPUQuantization, D: DimensionalNeuronDefaults<Q>>(
    // Dimensions / density
    cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
    neurons_per_voxel: NumberNeuronsPerVoxel,
    // Uniform per-neuron values
    neuron_global_burst_index_of_last_firing: BurstGlobalIndex<Q::BurstIndex>,
    neuron_membrane_potential: NPUNeuronMembranePotential<Q::Value>,
    neuron_fire_threshold: FireThreshold<Q::Value>,
    neuron_leak_coefficient: LeakCoefficient<Q::Percentage>,
    neuron_refractory_countdown: BurstDelta<Q::BurstDelta>,
    neuron_consecutive_fire_count: BurstDelta<Q::BurstDelta>,
    // Cortical-area-wide values
    cortical_excitability: NeuronExcitability<Q::Percentage>,
    cortical_refractory_period_limit: BurstDelta<Q::BurstDelta>,
    cortical_fire_threshold_limit: FireThresholdLimit<Q::Value>,
    cortical_consecutive_fire_limit: BurstDelta<Q::BurstDelta>,
    cortical_is_mp_charge_accumulation_enabled: bool,
    cortical_is_mp_driven_psp_enabled: bool,
    // Per-neuron storage
    neuron_cortical_area_indexes: &mut Vec<CorticalAreaIndex<Q::CorticalIndex>>,
    neuron_global_burst_indexes_of_last_firing: &mut Vec<BurstGlobalIndex<Q::BurstIndex>>,
    neuron_membrane_potentials: &mut Vec<NPUNeuronMembranePotential<Q::Value>>,
    neuron_fire_thresholds: &mut Vec<FireThreshold<Q::Value>>,
    neuron_leak_coefficients: &mut Vec<LeakCoefficient<Q::Percentage>>,
    neuron_flags: &mut Vec<NeuronFlag>,
    neuron_refractory_countdowns: &mut Vec<BurstDelta<Q::BurstDelta>>,
    neuron_consecutive_fire_counts: &mut Vec<BurstDelta<Q::BurstDelta>>,
    // Per-cortical-area storage
    cortical_datas: &mut IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>,
    // Cache of invalid (reusable) neuron index ranges
    cache_invalid_neuron_index_blocks: &mut RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>, NeuronCount<Q::NeuronIndex>>,
) -> Result<(CorticalAreaIndex<Q::CorticalIndex>, bool), FeagiNPUNeuronError> {

    let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
    let neuron_writing_region = cache_invalid_neuron_index_blocks.find_space(NeuronCount::from_usize(number_of_neurons));

    // NOTE: for now neuron flag only checks for validity, so we dont need that parameter.
    let mut cortical_flags: DimensionalNeuronCorticalFlag = DimensionalNeuronCorticalFlag::new_valid();
    cortical_flags.set_mp_charge_accumulation_enabled(cortical_is_mp_charge_accumulation_enabled);
    cortical_flags.set_mp_driven_psp_enabled(cortical_is_mp_driven_psp_enabled);

    let neuron_flag = NeuronFlag::new_valid();

    // TODO use par iter on massive arrays!

    let (output_cortical_index, extending) = match neuron_writing_region {
        None => {
            // No space, allocate at the end of the arrays
            let neuron_writing_region = NPUNeuronIndex::from_usize(neuron_flags.len()) .. NPUNeuronIndex::from_usize(neuron_flags.len() + number_of_neurons);
            let cortical_data = DimensionalNeuronCorticalData {
                flags: cortical_flags,
                neuron_range: neuron_writing_region.clone(),
                number_neurons_invalid_from_degeneration: NeuronCount::ZERO, // no neurons assumed dead yet
                dimensions: cortical_area_dimensions,
                number_neurons_per_voxel: neurons_per_voxel,
                excitability: cortical_excitability,
                refractory_period_limit: cortical_refractory_period_limit,
                fire_threshold_limit: cortical_fire_threshold_limit,
                consecutive_fire_limit: cortical_consecutive_fire_limit,
            };
            let cortical_index = cortical_datas.insert_data_and_get_unique_index(cortical_data);

            neuron_cortical_area_indexes.extend(core::iter::repeat_n(cortical_index, number_of_neurons));
            neuron_global_burst_indexes_of_last_firing.extend(core::iter::repeat_n(neuron_global_burst_index_of_last_firing, number_of_neurons));
            neuron_membrane_potentials.extend(core::iter::repeat_n(neuron_membrane_potential, number_of_neurons));
            neuron_fire_thresholds.extend(core::iter::repeat_n(neuron_fire_threshold, number_of_neurons));
            neuron_leak_coefficients.extend(core::iter::repeat_n(neuron_leak_coefficient, number_of_neurons));
            neuron_flags.extend(core::iter::repeat_n(neuron_flag, number_of_neurons));
            neuron_refractory_countdowns.extend(core::iter::repeat_n(neuron_refractory_countdown, number_of_neurons));
            neuron_consecutive_fire_counts.extend(core::iter::repeat_n(neuron_consecutive_fire_count, number_of_neurons));

            (cortical_index, true)
        }
        Some(neuron_writing_region) => {
            // We have space, overwrite the previously invalidated region
            let neuron_range_usize: Range<usize> = NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(neuron_writing_region.clone());
            let cortical_data = DimensionalNeuronCorticalData {
                flags: cortical_flags,
                neuron_range: neuron_writing_region.clone(),
                number_neurons_invalid_from_degeneration: NeuronCount::ZERO, // no neurons assumed dead yet
                dimensions: cortical_area_dimensions,
                number_neurons_per_voxel: neurons_per_voxel,
                excitability: cortical_excitability,
                refractory_period_limit: cortical_refractory_period_limit,
                fire_threshold_limit: cortical_fire_threshold_limit,
                consecutive_fire_limit: cortical_consecutive_fire_limit,
            };
            let cortical_index = cortical_datas.insert_data_and_get_unique_index(cortical_data);

            neuron_cortical_area_indexes[neuron_range_usize.clone()].fill(cortical_index);
            neuron_global_burst_indexes_of_last_firing[neuron_range_usize.clone()].fill(neuron_global_burst_index_of_last_firing);
            neuron_membrane_potentials[neuron_range_usize.clone()].fill(neuron_membrane_potential);
            neuron_fire_thresholds[neuron_range_usize.clone()].fill(neuron_fire_threshold);
            neuron_leak_coefficients[neuron_range_usize.clone()].fill(neuron_leak_coefficient);
            neuron_flags[neuron_range_usize.clone()].fill(neuron_flag);
            neuron_refractory_countdowns[neuron_range_usize.clone()].fill(neuron_refractory_countdown);
            neuron_consecutive_fire_counts[neuron_range_usize].fill(neuron_consecutive_fire_count);

            (cortical_index, false)
        }
    };

    Ok((output_cortical_index, extending))
}


/// Creates a cortical area in RAM-backed parallel neuron arrays, filling each neuron from the
/// pre-populated per-neuron values in `neuron_data`. Attempts to reuse a previously invalidated
/// region if one is big enough, otherwise appends to the end of the neuron arrays.
///
/// Returns the assigned cortical area index and the range of neuron indexes it covers and a bool for if it had to extend the arrays.
pub(crate) fn create_cortical_area_with_individualized_neurons<Q: NPUQuantization, D: DimensionalNeuronDefaults<Q>>(
    cortical_area_dimensions: NeuronVoxelDimensions<Q::Coord>,
    neurons_per_voxel: NumberNeuronsPerVoxel,
    neuron_data: DimensionalNeuronDataFromCorticalArea<Q>,
    // Per-neuron storage
    neuron_cortical_area_indexes: &mut Vec<CorticalAreaIndex<Q::CorticalIndex>>,
    neuron_global_burst_indexes_of_last_firing: &mut Vec<BurstGlobalIndex<Q::BurstIndex>>,
    neuron_membrane_potentials: &mut Vec<NPUNeuronMembranePotential<Q::Value>>,
    neuron_fire_thresholds: &mut Vec<FireThreshold<Q::Value>>,
    neuron_leak_coefficients: &mut Vec<LeakCoefficient<Q::Percentage>>,
    neuron_flags: &mut Vec<NeuronFlag>,
    neuron_refractory_countdowns: &mut Vec<BurstDelta<Q::BurstDelta>>,
    neuron_consecutive_fire_counts: &mut Vec<BurstDelta<Q::BurstDelta>>,
    // Per-cortical-area storage
    cortical_datas: &mut IndexedDataTracker<DimensionalNeuronCorticalData<Q>, CorticalAreaIndex<Q::CorticalIndex>>,
    // Cache of invalid (reusable) neuron index ranges
    cache_invalid_neuron_index_blocks: &mut RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>, NeuronCount<Q::NeuronIndex>>,
) -> Result<(CorticalAreaIndex<Q::CorticalIndex>, bool), FeagiNPUNeuronError> {

    let number_of_neurons: usize = cortical_area_dimensions.get_number_neurons(neurons_per_voxel);
    let neuron_writing_region = cache_invalid_neuron_index_blocks.find_space(NeuronCount::from_usize(number_of_neurons));

    let (output_cortical_index, extending) = match neuron_writing_region {
        None => {
            // No space, allocate at the end of the arrays
            let neuron_writing_region = NPUNeuronIndex::from_usize(neuron_flags.len()) .. NPUNeuronIndex::from_usize(neuron_flags.len() + number_of_neurons);
            let cortical_data = DimensionalNeuronCorticalData::new_default_valid::<D>(
                neuron_writing_region.clone(),
                cortical_area_dimensions,
                neurons_per_voxel,
            );
            let cortical_index = cortical_datas.insert_data_and_get_unique_index(cortical_data);

            neuron_cortical_area_indexes.extend(core::iter::repeat_n(cortical_index, number_of_neurons));
            neuron_global_burst_indexes_of_last_firing.extend(neuron_data.neuron_global_burst_index_of_last_firing);
            neuron_membrane_potentials.extend(neuron_data.neuron_membrane_potential);
            neuron_fire_thresholds.extend(neuron_data.neuron_fire_threshold);
            neuron_leak_coefficients.extend(neuron_data.neuron_leak_coefficient);
            neuron_flags.extend(neuron_data.neuron_flags);
            neuron_refractory_countdowns.extend(neuron_data.neuron_refractory_countdown);
            neuron_consecutive_fire_counts.extend(neuron_data.neuron_consecutive_fire_count);

            (cortical_index, true)
        }
        Some(neuron_writing_region) => {
            // We have space, overwrite the previously invalidated region
            let neuron_range_usize: Range<usize> = NPUNeuronIndex::<Q::NeuronIndex>::to_usize_range(neuron_writing_region.clone());
            let cortical_data = DimensionalNeuronCorticalData::new_default_valid::<D>(
                neuron_writing_region.clone(),
                cortical_area_dimensions,
                neurons_per_voxel,
            );
            let cortical_index = cortical_datas.insert_data_and_get_unique_index(cortical_data);

            neuron_cortical_area_indexes[neuron_range_usize.clone()].fill(cortical_index);
            neuron_global_burst_indexes_of_last_firing[neuron_range_usize.clone()].copy_from_slice(&neuron_data.neuron_global_burst_index_of_last_firing);
            neuron_membrane_potentials[neuron_range_usize.clone()].copy_from_slice(&neuron_data.neuron_membrane_potential);
            neuron_fire_thresholds[neuron_range_usize.clone()].copy_from_slice(&neuron_data.neuron_fire_threshold);
            neuron_leak_coefficients[neuron_range_usize.clone()].copy_from_slice(&neuron_data.neuron_leak_coefficient);
            neuron_flags[neuron_range_usize.clone()].copy_from_slice(&neuron_data.neuron_flags);
            neuron_refractory_countdowns[neuron_range_usize.clone()].copy_from_slice(&neuron_data.neuron_refractory_countdown);
            neuron_consecutive_fire_counts[neuron_range_usize].copy_from_slice(&neuron_data.neuron_consecutive_fire_count);

            (cortical_index, false)
        }
    };

    Ok((output_cortical_index, extending))
}
