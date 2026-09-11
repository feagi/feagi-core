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

/// Linear signed percentage on a **single** X column per channel: `z = 0` decodes to **+1.0**,
/// `z = z_max_depth - 1` decodes to **-1.0**, with averaging when multiple spikes are present.
///
/// This layout matches motor cortical_area units such as one-wide signed `1×1×N` per device (e.g.
/// RotaryMotor with linear positioning).
#[inline]
pub(crate) fn decode_signed_percentage_from_linear_neurons_along_z(
    neuron_indexes_along_z: &[u32],
    z_max_depth: u32,
    replace_val: &mut SignedPercentage,
) {
    if neuron_indexes_along_z.is_empty() {
        replace_val.inplace_update_unchecked(0.0);
        return;
    }
    let max_idx = z_max_depth.saturating_sub(1) as f32;
    if max_idx <= 0.0 {
        replace_val.inplace_update_unchecked(0.0);
        return;
    }
    let avg_z = neuron_indexes_along_z.iter().copied().sum::<u32>() as f32
        / neuron_indexes_along_z.len() as f32;
    let v = 1.0 - 2.0 * (avg_z / max_idx);
    replace_val.inplace_update_unchecked(v.clamp(-1.0, 1.0));
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
    // Linear inverted mapping: val=1.0 -> idx 0, val=0.0 -> idx z_len-1.
    // The raw `floor((1 - val) * z_len)` yields `z_len` for val=0.0, which is
    // one past the last valid neuron and would silently fail to fire any neuron
    // in the live cortical_area area. Clamp to `z_len - 1` so the boundary case
    // still produces a real spike. Discretization implies a residual error of
    // up to `1 / z_len` on the val=0.0 boundary at decode time.
    neuron_indexes_along_z.clear();
    let max_idx = (z_length_as_float as u32).saturating_sub(1);
    let raw_idx = ((1.0 - val.get_as_0_1()) * z_length_as_float).floor() as u32;
    neuron_indexes_along_z.push(raw_idx.min(max_idx));
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
    // Linear inverted mapping per lobe: |val|=1.0 -> idx 0, |val|->0+ -> idx z_len-1.
    // Two distinct historical bugs are fixed here:
    //   1) `floor((1 - |val|) * z_len)` yielded `z_len` for |val| -> 0+, one past
    //      the last valid neuron, so the live cortical_area area silently failed to fire.
    //      Clamp to `z_len - 1`.
    //   2) The negative branch used `(-1.0 - (-val)) * z_len`, which collapses every
    //      negative value to a (saturating-cast) `0` regardless of magnitude. The
    //      correct mirrored formula is `(1.0 - (-val)) * z_len` so negative
    //      magnitudes encode along the negative lobe symmetrically with the positive
    //      lobe.
    // For exact zero, fire the smallest position on both lobes so the linear decoder
    // sums to zero (mirrors the fractional encoder's existing behavior at val=0).
    neuron_indexes_along_z_positive.clear();
    neuron_indexes_along_z_negative.clear();
    let max_idx = (z_length_as_float as u32).saturating_sub(1);
    let v = val.get_as_m1_1();
    if v == 0.0 {
        neuron_indexes_along_z_positive.push(max_idx);
        neuron_indexes_along_z_negative.push(max_idx);
    } else if v > 0.0 {
        let raw_idx = ((1.0 - v) * z_length_as_float).floor() as u32;
        neuron_indexes_along_z_positive.push(raw_idx.min(max_idx));
    } else {
        let raw_idx = ((1.0 - (-v)) * z_length_as_float).floor() as u32;
        neuron_indexes_along_z_negative.push(raw_idx.min(max_idx));
    }
}

/// Encode [`SignedPercentage`] for [`decode_signed_percentage_from_linear_neurons_along_z`]:
/// `v = +1` → `z = 0`, `v = -1` → `z = z_len - 1`.
#[inline]
pub(crate) fn encode_signed_percentage_to_linear_neuron_z_index_along_z(
    val: &SignedPercentage,
    z_length_as_float: f32,
    neuron_indexes_along_z: &mut Vec<u32>,
) {
    neuron_indexes_along_z.clear();
    let max_idx = (z_length_as_float as u32).saturating_sub(1);
    let v = val.get_as_m1_1().clamp(-1.0, 1.0);
    let raw_idx = ((1.0 - v) / 2.0 * max_idx as f32).floor() as u32;
    neuron_indexes_along_z.push(raw_idx.min(max_idx));
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
                neuron_indexes[0],
                z_max_depth - 1,
                "Value 0.0 should clamp to last valid z index (z_len-1) so a real neuron fires"
            );

            decode_unsigned_percentage_from_linear_neurons(
                &neuron_indexes,
                z_max_depth,
                &mut percentage,
            );
            // Discretization residual at the val=0 boundary is bounded by 1/z_len.
            let boundary_tolerance = (1.0 / z_max_depth_float) + tolerance;
            assert!(
                (percentage.get_as_0_1() - 0.0).abs() <= boundary_tolerance,
                "Round trip should preserve 0.0 within one bin (got {})",
                percentage.get_as_0_1()
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
            // Zero now fires the smallest position on both lobes (mirrors the
            // fractional encoder), so the linear decoder cancels the two
            // contributions back to exactly zero.
            assert_eq!(
                neuron_indexes_pos.len(),
                1,
                "Zero should have one positive neuron at the smallest position"
            );
            assert_eq!(
                neuron_indexes_neg.len(),
                1,
                "Zero should also have one negative neuron at the smallest position"
            );
            assert_eq!(
                neuron_indexes_pos[0],
                z_max_depth - 1,
                "Zero positive lobe should land on the last valid z index"
            );
            assert_eq!(
                neuron_indexes_neg[0],
                z_max_depth - 1,
                "Zero negative lobe should land on the last valid z index"
            );

            decode_signed_percentage_from_linear_neurons(
                &neuron_indexes_pos,
                &neuron_indexes_neg,
                z_max_depth,
                &mut percentage,
            );
            assert!(
                (percentage.get_as_m1_1() - 0.0).abs() < tolerance,
                "Round trip should preserve 0.0 (got {})",
                percentage.get_as_m1_1()
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

        //region Linear signed along-Z (one X column per channel, e.g. RotaryMotor template 1x1x9)

        let z_along_z: u32 = 9;
        let z_along_z_float: f32 = z_along_z as f32;

        {
            let mut p = SignedPercentage::new_from_m1_1(0.0).unwrap();
            decode_signed_percentage_from_linear_neurons_along_z(&[0], z_along_z, &mut p);
            assert!(
                (p.get_as_m1_1() - 1.0).abs() < tolerance,
                "z=0 should decode to +1.0"
            );
        }
        {
            let mut p = SignedPercentage::new_from_m1_1(0.0).unwrap();
            decode_signed_percentage_from_linear_neurons_along_z(&[8], z_along_z, &mut p);
            assert!(
                (p.get_as_m1_1() - (-1.0)).abs() < tolerance,
                "z=8 (depth 9) should decode to -1.0"
            );
        }
        {
            let mut p = SignedPercentage::new_from_m1_1(1.0).unwrap();
            let mut zs: Vec<u32> = Vec::new();
            encode_signed_percentage_to_linear_neuron_z_index_along_z(&p, z_along_z_float, &mut zs);
            assert_eq!(zs, vec![0]);
            decode_signed_percentage_from_linear_neurons_along_z(&zs, z_along_z, &mut p);
            assert!((p.get_as_m1_1() - 1.0).abs() < tolerance, "round-trip +1.0");
        }
        {
            let mut p = SignedPercentage::new_from_m1_1(-1.0).unwrap();
            let mut zs: Vec<u32> = Vec::new();
            encode_signed_percentage_to_linear_neuron_z_index_along_z(&p, z_along_z_float, &mut zs);
            assert_eq!(zs, vec![8]);
            decode_signed_percentage_from_linear_neurons_along_z(&zs, z_along_z, &mut p);
            assert!(
                (p.get_as_m1_1() - (-1.0)).abs() < tolerance,
                "round-trip -1.0"
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

    /// Regression: the unsigned linear encoder must never produce a z index that
    /// is past `z_len - 1`. The pre-fix implementation returned `z_len` for
    /// val=0.0, which would silently fail to fire any neuron in the live
    /// cortical_area area, yielding an asymmetric IPU (fires at val=1.0, silent at
    /// val=0.0).
    #[test]
    fn unsigned_linear_encoder_never_exceeds_max_z_index() {
        let z_len: u32 = 10;
        let z_len_f = z_len as f32;
        for hundredth in 0u32..=100 {
            let v = hundredth as f32 / 100.0;
            let percentage = Percentage::new_from_0_1_unchecked(v);
            let mut indexes: Vec<u32> = Vec::new();
            encode_unsigned_percentage_to_linear_neuron_z_index(&percentage, z_len_f, &mut indexes);
            assert_eq!(indexes.len(), 1, "exactly one neuron per scalar sample");
            assert!(
                indexes[0] < z_len,
                "z index {} must be < z_len {} for val={}",
                indexes[0],
                z_len,
                v
            );
        }
    }

    /// Regression: the signed linear encoder's negative branch used
    /// `(-1.0 - (-v)) * z_len` which produces a negative float that
    /// saturating-casts to `0` for every negative magnitude. The fix should
    /// give a faithful, monotonically increasing z index as |val| decreases
    /// from 1.0 toward 0+.
    #[test]
    fn signed_linear_encoder_negative_branch_is_monotonic() {
        let z_len: u32 = 10;
        let z_len_f = z_len as f32;
        let mut last_idx: Option<u32> = None;
        // Sweep from |v|=1.0 down toward 0+; the active z index should be
        // monotonically non-decreasing (closer-to-zero -> larger index).
        for tenth in (1u32..=10).rev() {
            let v = -(tenth as f32 / 10.0);
            let percentage = SignedPercentage::new_from_m1_1_unchecked(v);
            let mut pos: Vec<u32> = Vec::new();
            let mut neg: Vec<u32> = Vec::new();
            encode_signed_percentage_to_linear_neuron_z_index(
                &percentage,
                z_len_f,
                &mut pos,
                &mut neg,
            );
            assert!(
                pos.is_empty(),
                "negative magnitude must not write the positive lobe (v={})",
                v
            );
            assert_eq!(
                neg.len(),
                1,
                "negative magnitude must fire exactly one neuron (v={})",
                v
            );
            let idx = neg[0];
            assert!(
                idx < z_len,
                "z index {} must be < z_len {} for v={}",
                idx,
                z_len,
                v
            );
            if let Some(prev) = last_idx {
                assert!(
                    idx >= prev,
                    "z index must be monotonically non-decreasing as |v| -> 0; got {} after {} (v={})",
                    idx, prev, v
                );
            }
            last_idx = Some(idx);
        }
    }

    /// Regression: positive and negative magnitudes of equal absolute value
    /// must occupy mirrored z indices on their respective lobes. Pre-fix the
    /// negative branch was broken so this property did not hold.
    #[test]
    fn signed_linear_encoder_is_lobe_symmetric() {
        let z_len: u32 = 10;
        let z_len_f = z_len as f32;
        for tenth in 1u32..=10 {
            let mag = tenth as f32 / 10.0;
            let pos_val = SignedPercentage::new_from_m1_1_unchecked(mag);
            let neg_val = SignedPercentage::new_from_m1_1_unchecked(-mag);

            let mut pos_pos: Vec<u32> = Vec::new();
            let mut pos_neg: Vec<u32> = Vec::new();
            encode_signed_percentage_to_linear_neuron_z_index(
                &pos_val,
                z_len_f,
                &mut pos_pos,
                &mut pos_neg,
            );

            let mut neg_pos: Vec<u32> = Vec::new();
            let mut neg_neg: Vec<u32> = Vec::new();
            encode_signed_percentage_to_linear_neuron_z_index(
                &neg_val,
                z_len_f,
                &mut neg_pos,
                &mut neg_neg,
            );

            assert!(pos_neg.is_empty() && neg_pos.is_empty());
            assert_eq!(
                pos_pos, neg_neg,
                "mirrored magnitudes must produce identical z indices on opposing lobes (mag={})",
                mag
            );
        }
    }

    /// Regression: round-trip preserves the value within a single discretization
    /// bin for both signed and unsigned linear encoders across the full range.
    #[test]
    fn linear_encoders_round_trip_within_one_bin() {
        let z_len: u32 = 16;
        let z_len_f = z_len as f32;
        let bin_tolerance = 1.0 / z_len_f + 1e-4;

        // Unsigned sweep
        for sixteenth in 0u32..=16 {
            let v = sixteenth as f32 / 16.0;
            let mut percentage = Percentage::new_from_0_1_unchecked(v);
            let mut indexes: Vec<u32> = Vec::new();
            encode_unsigned_percentage_to_linear_neuron_z_index(&percentage, z_len_f, &mut indexes);
            decode_unsigned_percentage_from_linear_neurons(&indexes, z_len, &mut percentage);
            assert!(
                (percentage.get_as_0_1() - v).abs() <= bin_tolerance,
                "unsigned round-trip should be within one bin for v={} (got {})",
                v,
                percentage.get_as_0_1()
            );
        }

        // Signed sweep covering both lobes (skip exactly +/-1 boundary which
        // cannot fall outside a single bin anyway).
        for sixteenth in -16i32..=16 {
            let v = sixteenth as f32 / 16.0;
            let mut percentage = SignedPercentage::new_from_m1_1_unchecked(v);
            let mut pos: Vec<u32> = Vec::new();
            let mut neg: Vec<u32> = Vec::new();
            encode_signed_percentage_to_linear_neuron_z_index(
                &percentage,
                z_len_f,
                &mut pos,
                &mut neg,
            );
            decode_signed_percentage_from_linear_neurons(&pos, &neg, z_len, &mut percentage);
            assert!(
                (percentage.get_as_m1_1() - v).abs() <= bin_tolerance,
                "signed round-trip should be within one bin for v={} (got {})",
                v,
                percentage.get_as_m1_1()
            );
        }
    }
}
