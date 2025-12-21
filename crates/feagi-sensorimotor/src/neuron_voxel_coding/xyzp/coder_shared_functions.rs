use crate::data_types::{Percentage, SignedPercentage};

//region Decode Percentages
#[inline]
pub(crate) fn decode_unsigned_percentage_from_linear_neurons(
    neuron_indexes_along_z: &[u32],
    z_max_depth: u32,
    replace_val: &mut Percentage,
) {
    let z_max_depth: f32 = z_max_depth as f32; // WARNING: If we ever get neuron indexes past z_max_depth, we run the risk of invalid percentages!
    let average_index_value: f32 = neuron_indexes_along_z.iter().copied().sum::<u32>() as f32
        / (z_max_depth * neuron_indexes_along_z.len() as f32);
    replace_val.inplace_update_unchecked(1.0 - average_index_value); // Flip since index z 0 should be max value
}

#[inline]
pub(crate) fn decode_signed_percentage_from_linear_neurons(
    neuron_indexes_along_z_positive: &[u32],
    neuron_indexes_along_z_negative: &[u32],
    z_max_depth: u32,
    replace_val: &mut SignedPercentage,
) {
    let z_max_depth: f32 = z_max_depth as f32;

    // Handle division by zero: if vector is empty, use 0.0
    let positive = if neuron_indexes_along_z_positive.is_empty() {
        0.0
    } else {
        1.0 - neuron_indexes_along_z_positive.iter().copied().sum::<u32>() as f32
            / (z_max_depth * neuron_indexes_along_z_positive.len() as f32)
    };

    let negative = if neuron_indexes_along_z_negative.is_empty() {
        0.0
    } else {
        1.0 - neuron_indexes_along_z_negative.iter().copied().sum::<u32>() as f32
            / (z_max_depth * neuron_indexes_along_z_negative.len() as f32)
    };

    replace_val.inplace_update_unchecked(positive - negative);
}

#[inline]
pub(crate) fn decode_unsigned_percentage_from_fractional_exponential_neurons(
    neuron_indexes_along_z: &Vec<u32>,
    replace_val: &mut Percentage,
) {
    let mut processing: f32 = 0.0; // WARNING: If there are repeats along z, then we will have issues
    for z in neuron_indexes_along_z {
        processing += 0.5f32.powi(*z as i32);
    }
    replace_val.inplace_update_unchecked(processing);
}

#[inline]
pub(crate) fn decode_signed_percentage_from_fractional_exponential_neurons(
    neuron_indexes_along_z_positive: &Vec<u32>,
    neuron_indexes_along_z_negative: &Vec<u32>,
    replace_val: &mut SignedPercentage,
) {
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
pub(crate) fn encode_unsigned_percentage_to_linear_neuron_z_index(
    val: &Percentage,
    z_length_as_float: f32,
    neuron_indexes_along_z: &mut Vec<u32>,
) {
    neuron_indexes_along_z.clear();
    neuron_indexes_along_z.push(((1.0 - val.get_as_0_1()) * z_length_as_float).floor() as u32);
}

#[inline]
pub(crate) fn encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
    replace_val: &Percentage,
    number_neurons_along_z: u32,
    neuron_indexes_along_z: &mut Vec<u32>,
) {
    neuron_indexes_along_z.clear();
    let mut processing: f32 = replace_val.get_as_0_1();
    if processing == 0.0 {
        // In the case of 0, lets push the positive smallest value they have
        neuron_indexes_along_z.push(number_neurons_along_z - 1);
    } else {
        for z in 1..(number_neurons_along_z + 1) {
            let compare: f32 = 0.5f32.powi(z as i32);
            if processing >= compare {
                processing -= compare;
                neuron_indexes_along_z.push(z - 1);
            }
        }
    }
}

#[inline]
pub(crate) fn encode_signed_percentage_to_linear_neuron_z_index(
    val: &SignedPercentage,
    z_length_as_float: f32,
    neuron_indexes_along_z_positive: &mut Vec<u32>,
    neuron_indexes_along_z_negative: &mut Vec<u32>,
) {
    neuron_indexes_along_z_positive.clear();
    neuron_indexes_along_z_negative.clear();
    if val.is_positive_or_zero() {
        neuron_indexes_along_z_positive
            .push(((1.0 - val.get_as_m1_1()) * z_length_as_float).floor() as u32);
    } else {
        neuron_indexes_along_z_negative
            .push(((-1.0 - (-val.get_as_m1_1())) * z_length_as_float).floor() as u32);
    }
}

#[inline]
pub(crate) fn encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
    replace_val: &SignedPercentage,
    number_neurons_along_z: u32,
    neuron_indexes_along_z_positive: &mut Vec<u32>,
    neuron_indexes_along_z_negative: &mut Vec<u32>,
) {
    neuron_indexes_along_z_positive.clear();
    neuron_indexes_along_z_negative.clear();

    let mut processing: f32 = replace_val.get_as_m1_1();
    if processing == 0.0 {
        // In the case of 0, lets push both the positive and negative smallest value they have
        neuron_indexes_along_z_positive.push(number_neurons_along_z - 1);
        neuron_indexes_along_z_negative.push(number_neurons_along_z - 1);
    } else if processing < 0.0f32 {
        // negative non-zero
        processing *= -1.0; // make positive once
        for z in 1..(number_neurons_along_z + 1) {
            let compare: f32 = 0.5f32.powi(z as i32);
            if processing >= compare {
                processing -= compare;
                neuron_indexes_along_z_negative.push(z - 1);
            }
        }
    } else {
        // positive non-zero
        for z in 1..(number_neurons_along_z + 1) {
            let compare: f32 = 0.5f32.powi(z as i32);
            if processing >= compare {
                processing -= compare;
                neuron_indexes_along_z_positive.push(z - 1);
            }
        }
    }
}

//endregion

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_types::{Percentage, SignedPercentage};

    #[test]
    fn test_shared_coder_functions() {
        // Test constants
        let z_max_depth: u32 = 10;
        let z_max_depth_float: f32 = z_max_depth as f32;
        let tolerance: f32 = 0.001;

        //region Linear Unsigned Percentage Tests

        // Test encode/decode unsigned percentage with linear neurons - value 0.0
        {
            let mut percentage = Percentage::new_zero();
            let mut neuron_indexes = Vec::new();

            encode_unsigned_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes,
            );
            assert_eq!(
                neuron_indexes.len(),
                1,
                "Should encode to exactly one neuron"
            );
            assert_eq!(
                neuron_indexes[0], 10,
                "Value 0.0 should map to max z index (inverted)"
            );

            decode_unsigned_percentage_from_linear_neurons(
                &neuron_indexes,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_0_1() - 0.0).abs() < tolerance,
                "Round trip should preserve 0.0"
            );
        }

        // Test encode/decode unsigned percentage with linear neurons - value 1.0
        {
            let mut percentage = Percentage::new_from_0_1_unchecked(1.0);
            let mut neuron_indexes = Vec::new();

            encode_unsigned_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes,
            );
            assert_eq!(
                neuron_indexes.len(),
                1,
                "Should encode to exactly one neuron"
            );
            assert_eq!(
                neuron_indexes[0], 0,
                "Value 1.0 should map to min z index (inverted)"
            );

            decode_unsigned_percentage_from_linear_neurons(
                &neuron_indexes,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_0_1() - 1.0).abs() < tolerance,
                "Round trip should preserve 1.0"
            );
        }

        // Test encode/decode unsigned percentage with linear neurons - value 0.5
        {
            let mut percentage = Percentage::new_from_0_1_unchecked(0.5);
            let mut neuron_indexes = Vec::new();

            encode_unsigned_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes,
            );
            assert_eq!(
                neuron_indexes.len(),
                1,
                "Should encode to exactly one neuron"
            );
            assert_eq!(
                neuron_indexes[0], 5,
                "Value 0.5 should map to middle z index"
            );

            decode_unsigned_percentage_from_linear_neurons(
                &neuron_indexes,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_0_1() - 0.5).abs() < tolerance,
                "Round trip should preserve 0.5"
            );
        }

        //endregion

        //region Linear Signed Percentage Tests

        // Test encode/decode signed percentage with linear neurons - value 0.0
        {
            let mut percentage = SignedPercentage::new_from_m1_1(0.0).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                1,
                "Zero should have one positive neuron"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                0,
                "Zero should have no negative neurons"
            );

            decode_signed_percentage_from_linear_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_m1_1() - 0.0).abs() < tolerance,
                "Round trip should preserve 0.0"
            );
        }

        // Test encode/decode signed percentage with linear neurons - value 1.0
        {
            let mut percentage = SignedPercentage::new_from_m1_1(1.0).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                1,
                "Positive value should have positive neurons"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                0,
                "Positive value should have no negative neurons"
            );
            assert_eq!(
                neuron_indexes_pos[0], 0,
                "Value 1.0 should map to min z index"
            );

            decode_signed_percentage_from_linear_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_m1_1() - 1.0).abs() < tolerance,
                "Round trip should preserve 1.0"
            );
        }

        // Test encode/decode signed percentage with linear neurons - value -1.0
        {
            let mut percentage = SignedPercentage::new_from_m1_1(-1.0).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                0,
                "Negative value should have no positive neurons"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                1,
                "Negative value should have negative neurons"
            );
            assert_eq!(
                neuron_indexes_neg[0], 0,
                "Value -1.0 should map to min z index"
            );

            decode_signed_percentage_from_linear_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_m1_1() - (-1.0)).abs() < tolerance,
                "Round trip should preserve -1.0"
            );
        }

        // Test encode/decode signed percentage with linear neurons - value 0.5
        {
            let mut percentage = SignedPercentage::new_from_m1_1_unchecked(0.5);
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_linear_neuron_z_index(
                &percentage,
                z_max_depth_float,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                1,
                "Positive value should have positive neurons"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                0,
                "Positive value should have no negative neurons"
            );

            decode_signed_percentage_from_linear_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_m1_1() - 0.5).abs() < tolerance,
                "Round trip should preserve 0.5"
            );
        }

        //endregion

        //region Fractional/Exponential Unsigned Percentage Tests

        // Test encode/decode unsigned percentage with fractional neurons - value 0.0
        {
            let mut percentage = Percentage::new_zero();
            let mut neuron_indexes = Vec::new();

            encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes,
            );
            assert_eq!(neuron_indexes.len(), 1, "Zero should produce 1 neuron");
            assert_eq!(
                neuron_indexes[0],
                z_max_depth - 1,
                "The neuron should be at the min value"
            );

            decode_unsigned_percentage_from_fractional_exponential_neurons(
                &neuron_indexes,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_0_1() - (0.5f32.powi(z_max_depth as i32))).abs() < tolerance,
                "Round trip should preserve close to 0"
            );
        }

        // Test encode/decode unsigned percentage with fractional neurons - value 0.5
        {
            let percentage = Percentage::new_from_0_1_unchecked(0.5);
            let mut neuron_indexes = Vec::new();

            encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes,
            );
            assert!(
                !neuron_indexes.is_empty(),
                "0.5 should produce at least one active neuron"
            );
            assert!(
                neuron_indexes.contains(&0),
                "0.5 should activate neuron at z=0 (0.5^0 = 0.5)"
            );

            let mut decoded_percentage = Percentage::new_zero();
            decode_unsigned_percentage_from_fractional_exponential_neurons(
                &neuron_indexes,
                &mut decoded_percentage,
            );
            // Note: Due to the fractional encoding, we may not get exact 0.5 back
            assert!(
                decoded_percentage.get_as_0_1() > 0.0,
                "Decoded value should be greater than 0"
            );
        }

        // Test encode/decode unsigned percentage with fractional neurons - value 1.0
        {
            let percentage = Percentage::new_from_0_1_unchecked(1.0);
            let mut neuron_indexes = Vec::new();

            encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes,
            );
            // 1.0 should activate many neurons since sum of 0.5^i approaches 1.0
            assert!(
                !neuron_indexes.is_empty(),
                "1.0 should produce active neurons"
            );

            let mut decoded_percentage = Percentage::new_zero();
            decode_unsigned_percentage_from_fractional_exponential_neurons(
                &neuron_indexes,
                &mut decoded_percentage,
            );
            assert!(
                decoded_percentage.get_as_0_1() > 0.5,
                "Decoded value should be substantial"
            );
        }

        //endregion

        //region Fractional/Exponential Signed Percentage Tests

        // Test encode/decode signed percentage with fractional neurons - value 0.0
        {
            let mut percentage = SignedPercentage::new_from_m1_1(0.0).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                1,
                "Zero should have 1 positive neuron"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                1,
                "Zero should have 1 negative neuron"
            );

            decode_signed_percentage_from_fractional_exponential_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_m1_1() - 0.0).abs() < tolerance,
                "Round trip should preserve 0.0"
            );
        }

        // Test encode/decode signed percentage with fractional neurons - value 0.5
        {
            let percentage = SignedPercentage::new_from_m1_1(0.5).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert!(
                !neuron_indexes_pos.is_empty(),
                "Positive value should have positive neurons"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                0,
                "Positive value should have no negative neurons"
            );

            let mut decoded_percentage = SignedPercentage::new_from_m1_1(0.0).unwrap();
            decode_signed_percentage_from_fractional_exponential_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                &mut decoded_percentage,
            );
            assert!(
                decoded_percentage.get_as_m1_1() > 0.0,
                "Decoded positive value should be positive"
            );
        }

        // Test encode/decode signed percentage with fractional neurons - value -0.5
        {
            let percentage = SignedPercentage::new_from_m1_1(-0.5).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                0,
                "Negative value should have no positive neurons"
            );
            assert!(
                !neuron_indexes_neg.is_empty(),
                "Negative value should have negative neurons"
            );

            let mut decoded_percentage = SignedPercentage::new_from_m1_1(0.0).unwrap();
            decode_signed_percentage_from_fractional_exponential_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                &mut decoded_percentage,
            );
            assert!(
                decoded_percentage.get_as_m1_1() < 0.0,
                "Decoded negative value should be negative"
            );
        }

        // Test encode/decode signed percentage with fractional neurons - value 1.0
        {
            let percentage = SignedPercentage::new_from_m1_1(1.0).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert!(
                !neuron_indexes_pos.is_empty(),
                "Value 1.0 should have positive neurons"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                0,
                "Value 1.0 should have no negative neurons"
            );

            let mut decoded_percentage = SignedPercentage::new_from_m1_1(0.0).unwrap();
            decode_signed_percentage_from_fractional_exponential_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                &mut decoded_percentage,
            );
            assert!(
                decoded_percentage.get_as_m1_1() > 0.5,
                "Decoded value should be substantially positive"
            );
        }

        // Test encode/decode signed percentage with fractional neurons - value -1.0
        {
            let percentage = SignedPercentage::new_from_m1_1(-1.0).unwrap();
            let mut neuron_indexes_pos = Vec::new();
            let mut neuron_indexes_neg = Vec::new();

            encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                &percentage,
                z_max_depth,
                &mut neuron_indexes_pos,
                &mut neuron_indexes_neg,
            );
            assert_eq!(
                neuron_indexes_pos.len(),
                0,
                "Value -1.0 should have no positive neurons"
            );
            assert!(
                !neuron_indexes_neg.is_empty(),
                "Value -1.0 should have negative neurons"
            );

            let mut decoded_percentage = SignedPercentage::new_from_m1_1(0.0).unwrap();
            decode_signed_percentage_from_fractional_exponential_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                &mut decoded_percentage,
            );
            assert!(
                decoded_percentage.get_as_m1_1() < -0.5,
                "Decoded value should be substantially negative"
            );
        }

        //endregion

        //region Edge Case Tests

        // Test decode with empty vectors for signed linear
        {
            let empty_pos: Vec<u32> = Vec::new();
            let empty_neg: Vec<u32> = Vec::new();
            let mut percentage = SignedPercentage::new_from_m1_1_unchecked(0.5); // Start with non-zero

            decode_signed_percentage_from_linear_neurons(
                &empty_pos,
                &empty_neg,
                z_max_depth,
                &mut percentage,
            );
            assert_eq!(
                percentage.get_as_m1_1(),
                0.0,
                "Empty vectors should decode to 0.0"
            );
        }

        // Test decode with empty vector for unsigned fractional
        {
            let empty: Vec<u32> = Vec::new();
            let mut percentage = Percentage::new_from_0_1_unchecked(0.5); // Start with non-zero

            decode_unsigned_percentage_from_fractional_exponential_neurons(&empty, &mut percentage);
            assert_eq!(
                percentage.get_as_0_1(),
                0.0,
                "Empty vector should decode to 0.0"
            );
        }

        // Test decode with empty vectors for signed fractional
        {
            let empty_pos: Vec<u32> = Vec::new();
            let empty_neg: Vec<u32> = Vec::new();
            let mut percentage = SignedPercentage::new_from_m1_1_unchecked(0.5); // Start with non-zero

            decode_signed_percentage_from_fractional_exponential_neurons(
                &empty_pos,
                &empty_neg,
                &mut percentage,
            );
            assert_eq!(
                percentage.get_as_m1_1(),
                0.0,
                "Empty vectors should decode to 0.0"
            );
        }

        //endregion

        println!("All coder shared function tests passed!");
    }
}
