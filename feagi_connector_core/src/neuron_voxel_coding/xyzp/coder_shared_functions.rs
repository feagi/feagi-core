use std::ops::Range;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::CorticalChannelCount;
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays};
use crate::data_types::Percentage;
use crate::wrapped_io_data::WrappedIOData;

#[inline]
pub(crate) fn decode_unsigned_binary_fractional_percentages(percentage_dimension_count: u32,
                                                            channel_count: CorticalChannelCount,
                                                            read_id: CorticalID,
                                                            read_target: &CorticalMappedXYZPNeuronVoxels,
                                                            z_depth: u32,
                                                            scratch_space: &mut Vec<Vec<u32>>,  // # channels * NUMBER_PAIRS_PER_CHANNEL long, basically 1 vector per 1 z rows
                                                            write_target: &mut Vec<WrappedIOData>,
                                                            channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError> {

    // NOTE: Expecting channel_changed to be all false. Do not reset write_target, we will write to it if we got a value for the channel!
    const ONLY_ALLOWED_Y: u32 = 0; // This structure never has height

    let neuron_array = read_target.get_neurons_of(&read_id);
    if neuron_array.is_none() {
        return Ok(()); // All false will be kept for channel_changed, no data modified
    }

    let mut neuron_array = neuron_array.unwrap();
    if neuron_array.is_empty() {
        return Ok(()); // This shouldn't happen?
    }

    for scratch_per_z_depth in scratch_space.iter_mut() { // Not worth making parallel
        scratch_per_z_depth.clear()
    }

    let max_possible_x_index = percentage_dimension_count * *channel_count; // Something is wrong if we reach here

    for neuron in neuron_array.iter() { // Due to array writing, this cannot practically be done parallel
        // Ignoring any neuron_voxels that have no potential (if sent for some reason).
        if neuron.cortical_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
            continue; // Something is wrong, but currently we will just skip these
        }

        if neuron.cortical_coordinate.x >= max_possible_x_index || neuron.cortical_coordinate.z >= z_depth {
            continue; // Something is wrong, but currently we will just skip these
        }

        let z_row_vector = scratch_space.get_mut(neuron.cortical_coordinate.x as usize).unwrap();
        z_row_vector.push(neuron.cortical_coordinate.z)
    };

    let z_depth_float = z_depth as f32;

    // At this point, we have numbers in scratch space to average out
    for channel_index in 0..*channel_count as usize { // Literally not worth making parallel... right?
        let z_row_a_index = channel_index * percentage_dimension_count as usize;

        // We need to ensure if ANY of the numbers changed (as in they added anything to the vector for that row that only originally had 0), we update it and label it as such
        for i in 0..percentage_dimension_count {
            
        }

    };
}


//region Percentage Binary Fractional
#[inline]
pub(crate) fn encode_unsigned_binary_fractional(x_offset: u32, y_offset: u32, z_length: u32, value: Percentage, neuron_targets: &mut NeuronVoxelXYZPArrays) {
    let processing = value.get_as_0_1();
    let mut cache_neuron = NeuronVoxelXYZP::new(x_offset, y_offset, 0, 1.0);

    for i in (0..(z_length as i32)).rev().into_iter() {
        let weight = 0.5f32.powi(i);
        if processing >= weight {
            cache_neuron.cortical_coordinate.z = i as u32;
            neuron_targets.push(&cache_neuron)
        }
    }
}

/*
#[inline]
pub(crate) fn decode_unsigned_binary_fractional(x_offset: u32, y_offset: u32, neuron_targets: &NeuronVoxelXYZPArrays) -> Percentage {
    let mut processing: f32 = 0.0;
    let (x_vec, y_vec, z_vec, _p_vec) = neuron_targets.borrow_xyzp_vectors();
    let length = x_vec.len();

    // TODO we should be able to multistream this across multiple elements, this is very slow
    for i in 0..length {
        if x_vec[i] != x_offset {
            continue;
        }

        if y_vec[i] != y_offset {
            continue;
        }
        // Note: We don't care for the P value, just that neuron firing exists
        processing += 0.5f32.powi(z_vec[i] as i32 + 1);
    }
    Percentage::new_from_0_1_unchecked(processing)
}

 */

#[inline]
pub(crate) fn decode_unsigned_binary_fractional_multichannel(x_offsets: Range<u32>, y_offset: u32, neuron_targets: &NeuronVoxelXYZPArrays, write_data_to: &mut Vec<Percentage>) {
    // WARNING: Assumes x_offsets has the same number of elements as write_data_to. If not, a crash is likely!

    for percentage in write_data_to.iter_mut() {
        percentage.inplace_update(0.0);
    }

    let number_neurons = neuron_targets.len();
    let (x_vec, y_vec, z_vec, _p_vec) = neuron_targets.borrow_xyzp_vectors();

    for i in 0..number_neurons {
        if !x_offsets.contains(&x_vec[i]) {
            continue;
        }

        if y_vec[i] != y_offset {
            continue;
        }

        let write_index = (x_vec[i] - x_offsets.start) as usize;
        let mut perc = write_data_to.get_mut(write_index).unwrap();
        perc.inplace_update(perc.get_as_0_1() + 0.5f32.powi(z_vec[i] as i32 + 1));
    }

}




//endregion

//region Percentage Linear

#[inline]
pub(crate) fn encode_percentage_linear(x_offset: u32, y_offset: u32, z_length: u32, value: Percentage, neuron_xyzparrays: &mut NeuronVoxelXYZPArrays) {
    const POTENTIAL: f32 = 1.0;
    let processing = value.get_as_0_1();
    let z_neuron_index: u32 = (processing * (z_length as f32)) as u32;
    neuron_xyzparrays.push_raw(x_offset, y_offset, z_neuron_index, POTENTIAL);
}


#[inline]
pub(crate) fn decode_percentage_linear_multichannel(x_offsets: Range<u32>, y_offset: u32, z_length: u32, neuron_targets: &NeuronVoxelXYZPArrays, write_data_to: &mut Vec<Percentage>) {

    write_data_to.clear();

    let number_neurons = neuron_targets.len();
    let (x_vec, y_vec, z_vec, _p_vec) = neuron_targets.borrow_xyzp_vectors();
    let z_length_f: f32 = z_length as f32;

    let mut z_neurons_positions_normalized: Vec<Vec<f32>> = Vec::with_capacity(neuron_targets.len());
    for _i in 0..write_data_to.len() {
        z_neurons_positions_normalized.push(Vec::new());
    }

    for n in 0..number_neurons {

        if !x_offsets.contains(&x_vec[n]) {
            continue;
        }

        if y_vec[n] != y_offset {
            continue;
        }

        if neuron_targets.get_z(n).unwrap() >= z_length {
            continue; // This shouldn't be possible. Skip // TODO should we have an error instead?
        }

        let write_index = (x_vec[n] - x_offsets.start) as usize;
        z_neurons_positions_normalized[write_index].push(z_vec[n] as f32 / z_length_f);
    };

    for z_neuron_positions_normalized in z_neurons_positions_normalized.iter_mut() {
        let number_of_neurons = z_neuron_positions_normalized.len() as f32;
        let val = z_neuron_positions_normalized.iter().sum::<f32>() / number_of_neurons;
        write_data_to.push(Percentage::new_from_0_1_unchecked(val));
    }
}


//region
