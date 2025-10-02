use std::ops::Range;
use feagi_data_structures::neurons::xyzp::{NeuronXYZP, NeuronXYZPArrays};
use crate::data_types::Percentage;

//region Percentage Binary Fractional
#[inline]
pub(crate) fn encode_unsigned_binary_fractional(x_offset: u32, y_offset: u32, z_length: u32, value: Percentage, neuron_targets: &mut NeuronXYZPArrays) {
    let processing = value.get_as_0_1();
    let mut cache_neuron = NeuronXYZP::new(x_offset,y_offset,0,1.0);

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
pub(crate) fn decode_unsigned_binary_fractional(x_offset: u32, y_offset: u32, neuron_targets: &NeuronXYZPArrays) -> Percentage {
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
pub(crate) fn decode_unsigned_binary_fractional_multichannel(x_offsets: Range<u32>, y_offset: u32, neuron_targets: &NeuronXYZPArrays, write_data_to: &mut Vec<Percentage>) {
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
pub(crate) fn encode_percentage_linear(x_offset: u32, y_offset: u32, z_length: u32, value: Percentage, neuron_xyzparrays: &mut NeuronXYZPArrays) {
    const POTENTIAL: f32 = 1.0;
    let processing = value.get_as_0_1();
    let z_neuron_index: u32 = (processing * (z_length as f32)) as u32;
    neuron_xyzparrays.push_raw(x_offset, y_offset, z_neuron_index, POTENTIAL);
}


#[inline]
pub(crate) fn decode_percentage_linear_multichannel(x_offsets: Range<u32>, y_offset: u32, z_length: u32, neuron_targets: &NeuronXYZPArrays, write_data_to: &mut Vec<Percentage>) {

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
