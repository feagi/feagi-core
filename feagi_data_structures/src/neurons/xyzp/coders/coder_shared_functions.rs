use crate::data::Percentage;
use crate::neurons::xyzp::{NeuronXYZP, NeuronXYZPArrays};

#[inline]
pub(crate) fn encode_unsigned_binary_fractional(x_offset: u32, y_offset: u32, z_length: u32, value: Percentage, neuron_targets: &mut NeuronXYZPArrays) {

    neuron_targets.clear();
    let mut processing = value.get_as_0_1();
    let mut cache_neuron = NeuronXYZP::new(x_offset,y_offset,0,1.0);;

    for i in (0..(z_length as i32)).rev().into_iter() {
        let weight = 0.5f32.powi(i);
        if processing >= weight {
            cache_neuron.cortical_coordinate.z = i as u32;
            neuron_targets.push(&cache_neuron)
        }
    }
}


#[inline]
pub(crate) fn decode_unsigned_binary_fractional(x_offset: u32, y_offset: u32, z_offset: u32, neuron_targets: &NeuronXYZPArrays) -> Percentage {
    let mut processing: f32 = 0.0;
    let (x_vec, y_vec, z_vec, p_vec) = neuron_targets.borrow_xyzp_vectors();
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
        processing += 0.5f32.powi((z_offset - z_vec[i]) as i32);
    }
    Percentage::new_from_0_1_unchecked(processing)
}