use std::ops::Range;
use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZP, NeuronVoxelXYZPArrays};
use crate::data_types::{Percentage, SignedPercentage};

//region Decode Percentages
#[inline]
pub(crate) fn decode_unsigned_percentage_from_linear_neurons(neuron_indexes_along_z: &Vec<u32>, z_max_depth: u32, replace_val: &mut Percentage) {
    let z_max_depth: f32 = z_max_depth as f32; // WARNING: If we ever get neuron indexes past z_max_depth, we run the risk of invalid percentages!
    let average_index_value: f32 =  neuron_indexes_along_z.iter().copied().sum::<u32>() as f32 / (z_max_depth * neuron_indexes_along_z.len() as f32);
    replace_val.inplace_update_unchecked(1.0 - average_index_value); // Flip since index z 0 should be max value
}

#[inline]
pub(crate) fn decode_signed_percentage_from_linear_neurons(neuron_indexes_along_z_positive: &Vec<u32>, neuron_indexes_along_z_negative: &Vec<u32>, z_max_depth: u32, replace_val: &mut SignedPercentage) {
    let z_max_depth: f32 = z_max_depth as f32;
    
    // Handle division by zero: if vector is empty, use 0.0
    let positive = if neuron_indexes_along_z_positive.is_empty() {
        0.0
    } else {
        1.0 - neuron_indexes_along_z_positive.iter().copied().sum::<u32>() as f32 / (z_max_depth * neuron_indexes_along_z_positive.len() as f32)
    };
    
    let negative = if neuron_indexes_along_z_negative.is_empty() {
        0.0
    } else {
        1.0 - neuron_indexes_along_z_negative.iter().copied().sum::<u32>() as f32 / (z_max_depth * neuron_indexes_along_z_negative.len() as f32)
    };
    
    replace_val.inplace_update_unchecked(positive - negative);
}

#[inline]
pub(crate) fn decode_unsigned_percentage_from_fractional_exponential_neurons(neuron_indexes_along_z: &Vec<u32>, replace_val: &mut Percentage) {
    let mut processing: f32 = 0.0; // WARNING: If there are repeats along z, then we will have issues
     for z in neuron_indexes_along_z {
         processing += 0.5f32.powi(*z as i32);
     }
    replace_val.inplace_update_unchecked(processing);
}

#[inline]
pub(crate) fn decode_signed_percentage_from_fractional_exponential_neurons(neuron_indexes_along_z_positive: &Vec<u32>, neuron_indexes_along_z_negative: &Vec<u32>, replace_val: &mut SignedPercentage) {
    let mut processing: f32 = 0.0; // WARNING: If there are repeats along z, then we will have issues
    for z in neuron_indexes_along_z_positive {
        processing += 0.5f32.powi(*z as i32);
    }
    for z in neuron_indexes_along_z_negative {
        processing -= 0.5f32.powi(*z as i32);
    }
    replace_val.inplace_update_unchecked(processing);
}

//endregion

//region Encode Percentages
#[inline]
pub(crate) fn encode_unsigned_percentage_to_linear_neuron_z_index(val: &Percentage, z_length_as_float: f32, neuron_indexes_along_z: &mut Vec<u32>) {
    neuron_indexes_along_z.clear();
    neuron_indexes_along_z.push(((1.0 - val.get_as_0_1()) * z_length_as_float).floor() as u32);
}


#[inline]
pub(crate) fn encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(replace_val: &Percentage, number_neurons_along_z: u32, neuron_indexes_along_z: &mut Vec<u32>) {
    neuron_indexes_along_z.clear();
    let mut processing: f32 = replace_val.get_as_0_1();
    for z in 0..number_neurons_along_z {
        let compare: f32 = 0.5f32.powi(z as i32);
        if processing > compare {
            processing - compare;
            neuron_indexes_along_z.push(z);
        }
    }
}

#[inline]
pub(crate) fn encode_signed_percentage_to_linear_neuron_z_index(val: &SignedPercentage, z_length_as_float: f32, neuron_indexes_along_z_positive: &mut Vec<u32>, neuron_indexes_along_z_negative: &mut Vec<u32>) {
    neuron_indexes_along_z_positive.clear();
    neuron_indexes_along_z_negative.clear();
    if val.is_positive() {
        neuron_indexes_along_z_positive.push((val.get_as_m1_1() * z_length_as_float).floor() as u32);
    }
    else {
        neuron_indexes_along_z_negative.push((val.get_as_m1_1() * -1.0 * z_length_as_float).floor() as u32);
    }
}

#[inline]
pub(crate) fn encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(replace_val: &SignedPercentage, number_neurons_along_z: u32, neuron_indexes_along_z_positive: &mut Vec<u32>, neuron_indexes_along_z_negative: &mut Vec<u32>) {
    neuron_indexes_along_z_positive.clear();
    neuron_indexes_along_z_negative.clear();

    let mut processing: f32 = replace_val.get_as_m1_1();
    if processing < 0.0f32 {
        processing *= -1.0; // make positive
        for z in 0..number_neurons_along_z {
            let compare: f32 = 0.5f32.powi(z as i32);
            if processing > compare {
                processing - compare;
                neuron_indexes_along_z_negative.push(z);
            }
        }
        return;
    }
    else {
        for z in 0..number_neurons_along_z {
            let compare: f32 = 0.5f32.powi(z as i32);
            if processing > compare {
                processing - compare;
                neuron_indexes_along_z_positive.push(z);
            }
        }
        return;
    }
}


//endregion
