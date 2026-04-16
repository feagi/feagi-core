use core::ops::Range;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neurons::descriptors::NeuronCount;
use feagi_structures::useful_structs::{InvalidatableVector, RangeUintVector};
use crate::neuron::dimensional_neurons::shared_structs::DimensionalNeuronCorticalData;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUNeuronIndex, NPUQuantization};

/// Get the cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
pub(crate) fn get_cortical_area_ref<'a, Q: NPUQuantization>(cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>, cortical_data: &'a InvalidatableVector<DimensionalNeuronCorticalData<Q>>)
    -> Result<&'a DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError>
{
    Ok(cortical_data.get(cortical_area_index.to_usize())
        .ok_or_else(|| FeagiNPUNeuronError::InvalidCorticalIndex{
            context: "Requested Cortical Area Index does not exist!",
            given_cortical_index: cortical_area_index.to_usize() as u32
        })?)
}

/// Get the mutable cortical area properties by index. WARNING: AREA MAY EXIST BUT NOT BE VALID!
pub(crate) fn get_cortical_area_ref_mut<'a, Q: NPUQuantization>(cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>, cortical_data: &'a mut InvalidatableVector<DimensionalNeuronCorticalData<Q>>)
    -> Result<&'a mut DimensionalNeuronCorticalData<Q>, FeagiNPUNeuronError>
{
    Ok(cortical_data.get_mut(cortical_area_index.to_usize())
        .ok_or_else(|| FeagiNPUNeuronError::InvalidCorticalIndex{
            context: "Requested Cortical Area Index does not exist!",
            given_cortical_index: cortical_area_index.to_usize() as u32
        })?)
}


/// Marks the neurons of a cortical area as invalid, as well as other cache work in this regard.
/// Returns the range of neuron indexes invalidated.
pub(crate) fn invalidate_cortical_area_and_return_invalidated_neuron_range<Q: NPUQuantization>(
    cortical_area_index: &CorticalAreaIndex<Q::CorticalIndex>, 
    cortical_data: &mut InvalidatableVector<DimensionalNeuronCorticalData<Q>>, 
    neuron_flags: &mut Vec<NeuronFlag>, 
    number_valid_neurons: &mut NeuronCount<Q::NeuronIndex>, 
    number_invalid_neurons: &mut NeuronCount<Q::NeuronIndex>,
    invalid_neuron_index_blocks: &mut RangeUintVector<NPUNeuronIndex<Q::NeuronIndex>>)
    -> Result<Range<NPUNeuronIndex<Q::NeuronIndex>>, FeagiNPUNeuronError> {


    let cortical_data = get_cortical_area_ref_mut(&cortical_area_index, cortical_data)?;
    
    // so, since we actually do not care for any other flag in the neuron data except for
    // the is valid flag being set to false, just mass fill the area with the bitpack containing
    // that setting
    let neuron_flag_slice: &mut[NeuronFlag] = neuron_flags[cortical_data.neuron_range];
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