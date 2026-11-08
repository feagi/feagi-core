//! Single-voxel decoding for hover/inspection UIs.
//!
//! Uses the same decoding logic as feagi-sensorimotor's batch decoders so that
//! what Brain Visualizer displays matches what a robot/controller would process.

use crate::data_types::{Percentage, SignedPercentage};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    target_to_z_bin_signed_linear_along_z, target_to_z_bin_unsigned_fractional,
    target_to_z_bin_unsigned_linear,
};
pub use crate::neuron_voxel_coding::xyzp::percentage_variant::{
    PercentageCodingError, PercentageCodingVariant,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use std::fmt;

/// Result of decoding a single voxel.
#[derive(Debug, Clone)]
pub struct SingleVoxelDecodeResult {
    pub success: bool,
    pub channel: i32,
    pub value_percent: f32,
    pub value_0_1: f32,
    pub data_type: String,
    pub error: String,
}

impl SingleVoxelDecodeResult {
    pub fn ok(channel: i32, value_percent: f32, value_0_1: f32, data_type: &str) -> Self {
        Self {
            success: true,
            channel,
            value_percent,
            value_0_1,
            data_type: data_type.to_string(),
            error: String::new(),
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            channel: -1,
            value_percent: 0.0,
            value_0_1: 0.0,
            data_type: String::new(),
            error: msg.into(),
        }
    }
}

impl fmt::Display for SingleVoxelDecodeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            write!(
                f,
                "CH:{} {:.2}% ({})",
                self.channel, self.value_percent, self.data_type
            )
        } else {
            write!(f, "decode error: {}", self.error)
        }
    }
}

/// Dimensions of one channel's voxel block (from cortical area genome).
#[derive(Debug, Clone, Copy)]
pub struct ChannelDimensions {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl ChannelDimensions {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

/// Decode a single voxel using encoding from cortical area (genome).
///
/// Used when cortical ID has no binary config (e.g. legacy ASCII IDs). Caller provides
/// encoding from the cortical area. Uses the same formulas as the primary decode path.
///
/// # Arguments
/// * `encoding_type` - "linear" or "exponential" (fractional)
/// * `encoding_format` - "1d", "2d", "3d", or "4d"
/// * `is_signed` - true for SignedPercentage variants
#[allow(clippy::too_many_arguments)]
pub fn decode_single_voxel_from_encoding(
    encoding_type: &str,
    encoding_format: &str,
    is_signed: bool,
    voxel_x: u32,
    voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    device_count: u32,
) -> SingleVoxelDecodeResult {
    let positioning = match encoding_type.to_lowercase().as_str() {
        "exponential" | "fractional" => PercentageNeuronPositioning::Fractional,
        _ => PercentageNeuronPositioning::Linear,
    };
    let ndim = match encoding_format.to_lowercase().as_str() {
        "2d" => 2,
        "3d" => 3,
        "4d" => 4,
        _ => 1,
    };

    if is_signed {
        decode_percentage_signed(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            device_count.max(1),
            positioning,
            ndim,
        )
    } else {
        decode_percentage_unsigned(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            device_count.max(1),
            positioning,
            ndim,
        )
    }
}

/// Decode a single voxel (x, y, z) using cortical ID and channel dimensions.
///
/// Uses the exact same formulas as feagi-sensorimotor's batch decoders so that
/// Brain Visualizer hover display matches what a robot/controller would process.
///
/// # Arguments
/// * `cortical_id` - The cortical area ID (base64 or parsed)
/// * `voxel_x`, `voxel_y`, `voxel_z` - Voxel coordinates
/// * `channel_dims` - Dimensions per channel (from cortical_dimensions_per_device)
/// * `device_count` - Number of devices/channels (0 = treat as 1)
pub fn decode_single_voxel(
    cortical_id: &CorticalID,
    voxel_x: u32,
    voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    device_count: u32,
) -> SingleVoxelDecodeResult {
    let io_flag = match cortical_id.extract_io_data_flag() {
        Ok(f) => f,
        Err(e) => return SingleVoxelDecodeResult::err(format!("{}", e)),
    };

    let num_channels = if device_count > 0 { device_count } else { 1 };

    let z_depth = channel_dims.z;
    if z_depth == 0 {
        return SingleVoxelDecodeResult::err("channel z dimension must be > 0");
    }

    match io_flag {
        IOCorticalAreaConfigurationFlag::Boolean => {
            decode_boolean(voxel_z, z_depth, num_channels, channel_dims)
        }
        IOCorticalAreaConfigurationFlag::Percentage(_, pos) => decode_percentage_unsigned(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            1,
        ),
        IOCorticalAreaConfigurationFlag::Percentage2D(_, pos) => decode_percentage_unsigned(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            2,
        ),
        IOCorticalAreaConfigurationFlag::Percentage3D(_, pos) => decode_percentage_unsigned(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            3,
        ),
        IOCorticalAreaConfigurationFlag::Percentage4D(_, pos) => decode_percentage_unsigned(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            4,
        ),
        IOCorticalAreaConfigurationFlag::SignedPercentage(_, pos) => decode_percentage_signed(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            1,
        ),
        IOCorticalAreaConfigurationFlag::SignedPercentage2D(_, pos) => decode_percentage_signed(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            2,
        ),
        IOCorticalAreaConfigurationFlag::SignedPercentage3D(_, pos) => decode_percentage_signed(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            3,
        ),
        IOCorticalAreaConfigurationFlag::SignedPercentage4D(_, pos) => decode_percentage_signed(
            voxel_x,
            voxel_y,
            voxel_z,
            channel_dims,
            num_channels,
            pos,
            4,
        ),
        IOCorticalAreaConfigurationFlag::CartesianPlane(_) => {
            decode_cartesian_plane(voxel_x, voxel_y, voxel_z, channel_dims, num_channels)
        }
        IOCorticalAreaConfigurationFlag::Misc(_) => {
            decode_misc(voxel_x, voxel_y, voxel_z, channel_dims, num_channels)
        }
        IOCorticalAreaConfigurationFlag::PoseEstimation(..) => {
            decode_pose_estimation(voxel_x, voxel_y, voxel_z, channel_dims, num_channels)
        }
    }
}

fn decode_boolean(
    voxel_z: u32,
    z_depth: u32,
    num_channels: u32,
    channel_dims: ChannelDimensions,
) -> SingleVoxelDecodeResult {
    let ch_dim_x = channel_dims.x.max(1);
    let channel = (voxel_z / (z_depth * ch_dim_x)) as i32;
    if channel < 0 || channel >= num_channels as i32 {
        return SingleVoxelDecodeResult::err(format!(
            "channel {} out of range [0, {})",
            channel, num_channels
        ));
    }
    let local_z = voxel_z % z_depth;
    let value_0_1 = if local_z < z_depth / 2 { 0.0 } else { 1.0 };
    SingleVoxelDecodeResult::ok(channel, value_0_1 * 100.0, value_0_1, "Boolean")
}

fn decode_percentage_unsigned(
    voxel_x: u32,
    _voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    num_channels: u32,
    positioning: PercentageNeuronPositioning,
    ndim: u32,
) -> SingleVoxelDecodeResult {
    let ch_dim_x = channel_dims.x.max(1);
    let ch_dim_z = channel_dims.z;

    let x_per_channel = ch_dim_x * ndim;
    let channel = (voxel_x / x_per_channel) as i32;
    if channel < 0 || channel >= num_channels as i32 {
        return SingleVoxelDecodeResult::err(format!(
            "channel {} out of range [0, {})",
            channel, num_channels
        ));
    }
    if voxel_z >= ch_dim_z {
        return SingleVoxelDecodeResult::err(format!(
            "voxel_z {} >= z_depth {}",
            voxel_z, ch_dim_z
        ));
    }

    let mut percentage = Percentage::new_zero();
    let z_vec = vec![voxel_z];

    PercentageCodingVariant::from_parts(false, true, positioning).decode_unsigned_into(
        &z_vec,
        ch_dim_z,
        &mut percentage,
    );

    let value_0_1 = percentage.get_as_0_1();
    let pos_name = match positioning {
        PercentageNeuronPositioning::Linear => "Linear",
        PercentageNeuronPositioning::Fractional => "Fractional",
    };
    SingleVoxelDecodeResult::ok(
        channel,
        value_0_1 * 100.0,
        value_0_1,
        &format!("Percentage({})", pos_name),
    )
}

fn decode_percentage_signed(
    voxel_x: u32,
    voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    num_channels: u32,
    positioning: PercentageNeuronPositioning,
    ndim: u32,
) -> SingleVoxelDecodeResult {
    let ch_dim_x = channel_dims.x.max(1);
    let ch_dim_z = channel_dims.z;

    let is_incremental = ch_dim_x >= 2 || (channel_dims.y >= 2 && ndim == 1);
    let x_per_channel = if is_incremental { 2 } else { ch_dim_x * ndim };
    let channel = (voxel_x / x_per_channel) as i32;
    if channel < 0 || channel >= num_channels as i32 {
        return SingleVoxelDecodeResult::err(format!(
            "channel {} out of range [0, {})",
            channel, num_channels
        ));
    }
    if voxel_z >= ch_dim_z {
        return SingleVoxelDecodeResult::err(format!(
            "voxel_z {} >= z_depth {}",
            voxel_z, ch_dim_z
        ));
    }

    let z_vec = vec![voxel_z];
    let (z_pos, z_neg) = if ch_dim_x >= 2 {
        if voxel_x.is_multiple_of(2) {
            (z_vec.clone(), vec![])
        } else {
            (vec![], z_vec.clone())
        }
    } else if channel_dims.y >= 2 && ndim == 1 {
        if voxel_y == 0 {
            (z_vec.clone(), vec![])
        } else {
            (vec![], z_vec.clone())
        }
    } else {
        (z_vec.clone(), vec![])
    };

    let mut signed = SignedPercentage::new_from_m1_1(0.0)
        .unwrap_or_else(|_| SignedPercentage::new_from_m1_1_unchecked(0.0));

    // Variant from the shared selector. A genome layout that is wider than the one-column
    // along-z layout (X >= 2) is two-lobe even for a one-dimensional linear percentage.
    let variant = match PercentageCodingVariant::from_parts(true, ndim == 1, positioning) {
        PercentageCodingVariant::SignedLinearAlongZ if ch_dim_x != 1 => {
            PercentageCodingVariant::SignedLinearTwoLobe
        }
        selected => selected,
    };
    if variant == PercentageCodingVariant::SignedLinearAlongZ {
        variant.decode_signed_into(&z_vec, &[], ch_dim_z, &mut signed);
    } else {
        variant.decode_signed_into(&z_pos, &z_neg, ch_dim_z, &mut signed);
    }

    let value_m1_1 = signed.get_as_m1_1();
    let value_0_1 = (value_m1_1 + 1.0) / 2.0;
    let value_percent = value_m1_1 * 100.0;
    let pos_name = match positioning {
        PercentageNeuronPositioning::Linear => "Linear",
        PercentageNeuronPositioning::Fractional => "Fractional",
    };
    SingleVoxelDecodeResult::ok(
        channel,
        value_percent,
        value_0_1,
        &format!("SignedPercentage({})", pos_name),
    )
}

fn decode_cartesian_plane(
    voxel_x: u32,
    voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    num_channels: u32,
) -> SingleVoxelDecodeResult {
    let ch_dim_x = channel_dims.x.max(1);
    let ch_dim_y = channel_dims.y.max(1);
    let ch_dim_z = channel_dims.z.max(1);

    let channel = (voxel_x / ch_dim_x) as i32;
    if channel < 0 || channel >= num_channels as i32 {
        return SingleVoxelDecodeResult::err(format!(
            "channel {} out of range [0, {})",
            channel, num_channels
        ));
    }

    let local_x = voxel_x % ch_dim_x;
    let local_y = voxel_y % ch_dim_y;
    let x_norm = local_x as f32 / (ch_dim_x - 1) as f32;
    let y_norm = local_y as f32 / (ch_dim_y - 1) as f32;
    let z_norm = (voxel_z % ch_dim_z) as f32 / (ch_dim_z - 1) as f32;

    let value_percent = ((x_norm + y_norm + z_norm) / 3.0) * 100.0;
    SingleVoxelDecodeResult::ok(
        channel,
        value_percent,
        (x_norm + y_norm + z_norm) / 3.0,
        "CartesianPlane",
    )
}

fn decode_misc(
    voxel_x: u32,
    _voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    num_channels: u32,
) -> SingleVoxelDecodeResult {
    let ch_dim_x = channel_dims.x.max(1);
    let ch_dim_z = channel_dims.z.max(1);

    let channel = (voxel_x / ch_dim_x) as i32;
    if channel < 0 || channel >= num_channels as i32 {
        return SingleVoxelDecodeResult::err(format!(
            "channel {} out of range [0, {})",
            channel, num_channels
        ));
    }

    let local_z = voxel_z % ch_dim_z;
    let value_0_1 = local_z as f32 / (ch_dim_z - 1) as f32;
    SingleVoxelDecodeResult::ok(channel, value_0_1 * 100.0, value_0_1, "Misc")
}

fn decode_pose_estimation(
    voxel_x: u32,
    voxel_y: u32,
    voxel_z: u32,
    channel_dims: ChannelDimensions,
    num_channels: u32,
) -> SingleVoxelDecodeResult {
    let ch_dim_x = channel_dims.x.max(1);
    let ch_dim_y = channel_dims.y.max(1);
    let ch_dim_z = channel_dims.z.max(1);

    let channel = (voxel_x / ch_dim_x) as i32;
    if channel < 0 || channel >= num_channels as i32 {
        return SingleVoxelDecodeResult::err(format!(
            "channel {} out of range [0, {})",
            channel, num_channels
        ));
    }

    let local_x = voxel_x % ch_dim_x;
    let x_norm = local_x as f32 / (ch_dim_x - 1).max(1) as f32;
    let _y_norm = voxel_y as f32 / (ch_dim_y - 1).max(1) as f32;
    let joint_id = voxel_z % ch_dim_z;

    let value_percent = joint_id as f32 / (ch_dim_z - 1).max(1) as f32 * 100.0;
    SingleVoxelDecodeResult::ok(channel, value_percent, x_norm, "PoseEstimation")
}

//region Public percentage coder interface

/// Active z indexes of one encoded percentage value.
///
/// `negative` is filled only by the two-lobe variants (linear two-lobe and signed fractional);
/// for every other variant it is empty.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PercentageZIndexes {
    /// Active z indexes of the single column, or of the positive lobe.
    pub positive: Vec<u32>,
    /// Active z indexes of the negative lobe.
    pub negative: Vec<u32>,
}

/// Check a value is finite and inside `[min, max]`.
fn check_in_range(value: f64, min: f64, max: f64) -> Result<(), PercentageCodingError> {
    if value.is_finite() && value >= min && value <= max {
        Ok(())
    } else {
        Err(PercentageCodingError::ValueOutOfRange { value, min, max })
    }
}

/// Range of decoded values of a variant: `(0, 1)` unsigned, `(-1, 1)` signed.
fn value_range(variant: PercentageCodingVariant) -> (f64, f64) {
    if variant.is_signed() {
        (-1.0, 1.0)
    } else {
        (0.0, 1.0)
    }
}

/// Decode one percentage value from its active z indexes.
///
/// `depth` is the number of z bins of the channel. `z_positive` holds the active z indexes of
/// the single column (unsigned and signed along-z) or of the positive lobe; `z_negative` holds
/// the negative lobe and must be empty unless the variant is two-lobe. Returns the value in
/// the variant's native range (`0..=1` unsigned, `-1..=1` signed), computed by the same
/// functions as the batch decoders. Unsigned values need at least one active z index.
pub fn decode_percentage_for_variant(
    variant: PercentageCodingVariant,
    depth: u32,
    z_positive: &[u32],
    z_negative: &[u32],
) -> Result<f32, PercentageCodingError> {
    if depth == 0 {
        return Err(PercentageCodingError::ZeroDepth);
    }
    if !variant.uses_two_lobes() && !z_negative.is_empty() {
        return Err(PercentageCodingError::NegativeLobeNotApplicable(variant));
    }
    if let Some(&z) = z_positive.iter().chain(z_negative).find(|&&z| z >= depth) {
        return Err(PercentageCodingError::ZIndexOutOfRange { z, depth });
    }
    if variant.is_signed() {
        let mut signed = SignedPercentage::new_from_m1_1_unchecked(0.0);
        variant.decode_signed_into(z_positive, z_negative, depth, &mut signed);
        Ok(signed.get_as_m1_1())
    } else {
        if variant == PercentageCodingVariant::UnsignedLinear && z_positive.is_empty() {
            return Err(PercentageCodingError::NoActiveZIndex);
        }
        let mut unsigned = Percentage::new_zero();
        variant.decode_unsigned_into(z_positive, depth, &mut unsigned);
        Ok(unsigned.get_as_0_1())
    }
}

/// Decode one percentage value from its active z indexes, choosing the coding from the IO
/// configuration flag. See [`decode_percentage_for_variant`].
pub fn decode_percentage_from_z_indexes(
    flag: &IOCorticalAreaConfigurationFlag,
    depth: u32,
    z_positive: &[u32],
    z_negative: &[u32],
) -> Result<f32, PercentageCodingError> {
    decode_percentage_for_variant(
        PercentageCodingVariant::from_io_flag(flag)?,
        depth,
        z_positive,
        z_negative,
    )
}

/// Encode one percentage value to its active z indexes.
///
/// `value` must be finite and inside the variant's range (`0..=1` unsigned, `-1..=1` signed).
/// Uses the batch encoders' functions. The result decodes back only within about one bin
/// (`1 / (depth - 1)`): the linear encoder places a value at `floor((1 - v) * depth)` while the
/// linear decoder reads `1 - z / (depth - 1)`, so a round trip is not exact; use
/// [`target_to_z_bin`] when the z bin for a wanted decoded value is needed.
pub fn encode_percentage_for_variant(
    variant: PercentageCodingVariant,
    depth: u32,
    value: f32,
) -> Result<PercentageZIndexes, PercentageCodingError> {
    if depth == 0 {
        return Err(PercentageCodingError::ZeroDepth);
    }
    let (min, max) = value_range(variant);
    check_in_range(value as f64, min, max)?;
    let mut out = PercentageZIndexes::default();
    if variant.is_signed() {
        let signed = SignedPercentage::new_from_m1_1_unchecked(value);
        variant.encode_signed_into(&signed, depth, &mut out.positive, &mut out.negative);
    } else {
        let unsigned = Percentage::new_from_0_1_unchecked(value);
        variant.encode_unsigned_into(&unsigned, depth, &mut out.positive);
    }
    Ok(out)
}

/// Encode one percentage value to its active z indexes, choosing the coding from the IO
/// configuration flag. See [`encode_percentage_for_variant`].
pub fn encode_percentage_to_z_indexes(
    flag: &IOCorticalAreaConfigurationFlag,
    depth: u32,
    value: f32,
) -> Result<PercentageZIndexes, PercentageCodingError> {
    encode_percentage_for_variant(PercentageCodingVariant::from_io_flag(flag)?, depth, value)
}

/// The z bin whose decode is nearest to `target` (the inverse of the decoder).
///
/// * Unsigned linear: `round((1 - target) * (depth - 1))`, ties to even bin, computed in `f64`;
///   identical to the Python tooling's `value_to_z_bin`, so saved stance mappings do not shift.
/// * Signed along-z linear: `round((1 - target) / 2 * (depth - 1))`, ties to even.
/// * Unsigned fractional: the single bin whose decode `0.5^(z + 1)` is nearest (a fractional
///   value usually needs several bins; use [`encode_percentage_to_z_indexes`] for those).
/// * Signed two-lobe linear and signed fractional: an explicit
///   [`PercentageCodingError::NoSingleBinInverse`], since a value there needs a lobe as well as
///   a z bin.
///
/// `target` must be finite and inside the variant's range; it is not clamped. The decoder and
/// the encoder differ by up to one bin (see [`encode_percentage_for_variant`]); this function
/// inverts the decoder.
pub fn target_to_z_bin_for_variant(
    variant: PercentageCodingVariant,
    depth: u32,
    target: f64,
) -> Result<u32, PercentageCodingError> {
    if depth == 0 {
        return Err(PercentageCodingError::ZeroDepth);
    }
    let (min, max) = value_range(variant);
    check_in_range(target, min, max)?;
    match variant {
        PercentageCodingVariant::UnsignedLinear => {
            Ok(target_to_z_bin_unsigned_linear(target, depth))
        }
        PercentageCodingVariant::SignedLinearAlongZ => {
            Ok(target_to_z_bin_signed_linear_along_z(target, depth))
        }
        PercentageCodingVariant::UnsignedFractional => {
            Ok(target_to_z_bin_unsigned_fractional(target, depth))
        }
        PercentageCodingVariant::SignedLinearTwoLobe
        | PercentageCodingVariant::SignedFractional => {
            Err(PercentageCodingError::NoSingleBinInverse {
                variant,
                reason: "a signed two-lobe value needs a lobe as well as a z bin",
            })
        }
    }
}

/// The z bin whose decode is nearest to `target`, choosing the coding from the IO
/// configuration flag. See [`target_to_z_bin_for_variant`].
pub fn target_to_z_bin(
    flag: &IOCorticalAreaConfigurationFlag,
    depth: u32,
    target: f64,
) -> Result<u32, PercentageCodingError> {
    target_to_z_bin_for_variant(PercentageCodingVariant::from_io_flag(flag)?, depth, target)
}

//endregion

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, PercentageNeuronPositioning,
    };

    fn make_percentage_linear_id() -> CorticalID {
        let flag = IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Absolute,
            PercentageNeuronPositioning::Linear,
        );
        flag.as_io_cortical_id(false, *b"omo", 0u16.into(), 0u8.into())
    }

    fn make_percentage_fractional_id() -> CorticalID {
        let flag = IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Absolute,
            PercentageNeuronPositioning::Fractional,
        );
        flag.as_io_cortical_id(false, *b"omo", 0u16.into(), 0u8.into())
    }

    #[test]
    fn test_linear_percentage_z0_is_max() {
        let id = make_percentage_linear_id();
        let dims = ChannelDimensions::new(1, 1, 8);
        let r = decode_single_voxel(&id, 0, 0, 0, dims, 1);
        assert!(r.success);
        assert!(
            (r.value_0_1 - 1.0).abs() < 0.01,
            "z=0 should decode to 1.0 (max)"
        );
    }

    #[test]
    fn test_linear_percentage_zmax_is_min() {
        let id = make_percentage_linear_id();
        let dims = ChannelDimensions::new(1, 1, 8);
        let r = decode_single_voxel(&id, 0, 0, 7, dims, 1);
        assert!(r.success);
        assert!(
            (r.value_0_1 - 0.0).abs() < 0.01,
            "z=7 with z_max=8 -> 1.0 - 7/7 = 0.0 (min for linear)"
        );
    }

    #[test]
    fn test_exponential_percentage_z0() {
        let id = make_percentage_fractional_id();
        let dims = ChannelDimensions::new(1, 1, 8);
        let r = decode_single_voxel(&id, 0, 0, 0, dims, 1);
        assert!(r.success);
        assert!(
            (r.value_0_1 - 0.5).abs() < 0.01,
            "z=0 -> weight 0.5^(0+1) = 0.5 (largest single-neuron value, matches the encoder)"
        );
    }

    //region Public percentage coder interface tests

    const FRAME: FrameChangeHandling = FrameChangeHandling::Absolute;
    const LINEAR: PercentageNeuronPositioning = PercentageNeuronPositioning::Linear;
    const FRACTIONAL: PercentageNeuronPositioning = PercentageNeuronPositioning::Fractional;

    fn unsigned_flag(pos: PercentageNeuronPositioning) -> IOCorticalAreaConfigurationFlag {
        IOCorticalAreaConfigurationFlag::Percentage(FRAME, pos)
    }

    fn signed_flag(pos: PercentageNeuronPositioning) -> IOCorticalAreaConfigurationFlag {
        IOCorticalAreaConfigurationFlag::SignedPercentage(FRAME, pos)
    }

    fn signed_2d_flag(pos: PercentageNeuronPositioning) -> IOCorticalAreaConfigurationFlag {
        IOCorticalAreaConfigurationFlag::SignedPercentage2D(FRAME, pos)
    }

    /// `value_to_z_bin(v, 0, 1, depth)` of feagi-mcp `stance_mapping.py`, generated once with
    /// that function (Python `round`, ties to even) for the values in `INVERSE_VALUES`.
    const INVERSE_VALUES: [f64; 11] = [0.0, 0.1, 0.125, 0.25, 0.3, 0.4, 0.5, 0.6, 0.75, 0.9, 1.0];
    #[rustfmt::skip]
    const PYTHON_VALUE_TO_Z_BIN: [(u32, [u32; 11]); 8] = [
        (2, [1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0]),
        (3, [2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0]),
        (4, [3, 3, 3, 2, 2, 2, 2, 1, 1, 0, 0]),
        (6, [5, 4, 4, 4, 4, 3, 2, 2, 1, 0, 0]),
        (7, [6, 5, 5, 4, 4, 4, 3, 2, 2, 1, 0]),
        (9, [8, 7, 7, 6, 6, 5, 4, 3, 2, 1, 0]),
        (10, [9, 8, 8, 7, 6, 5, 4, 4, 2, 1, 0]),
        (16, [15, 14, 13, 11, 10, 9, 8, 6, 4, 1, 0]),
    ];

    #[test]
    fn selector_maps_flags_to_variants() {
        use PercentageCodingVariant as V;
        let cases = [
            (unsigned_flag(LINEAR), V::UnsignedLinear),
            (unsigned_flag(FRACTIONAL), V::UnsignedFractional),
            (signed_flag(LINEAR), V::SignedLinearAlongZ),
            (signed_flag(FRACTIONAL), V::SignedFractional),
            (signed_2d_flag(LINEAR), V::SignedLinearTwoLobe),
            (signed_2d_flag(FRACTIONAL), V::SignedFractional),
            (
                IOCorticalAreaConfigurationFlag::Percentage3D(FRAME, LINEAR),
                V::UnsignedLinear,
            ),
        ];
        for (flag, expected) in cases {
            assert_eq!(PercentageCodingVariant::from_io_flag(&flag), Ok(expected));
        }
    }

    #[test]
    fn selector_rejects_non_percentage_flag() {
        let err = PercentageCodingVariant::from_io_flag(&IOCorticalAreaConfigurationFlag::Boolean)
            .unwrap_err();
        assert!(matches!(err, PercentageCodingError::NotAPercentageFlag(_)));
        assert!(decode_percentage_from_z_indexes(
            &IOCorticalAreaConfigurationFlag::Boolean,
            8,
            &[0],
            &[]
        )
        .is_err());
    }

    #[test]
    fn decode_matches_single_voxel_decode_for_every_z() {
        // The public decode and the Brain Visualizer single-voxel decode share one selector.
        let depth = 9u32;
        let dims = ChannelDimensions::new(1, 1, depth);
        for pos in [LINEAR, FRACTIONAL] {
            let flag = unsigned_flag(pos);
            let id = flag.as_io_cortical_id(false, *b"omo", 0u16.into(), 0u8.into());
            for z in 0..depth {
                let voxel = decode_single_voxel(&id, 0, 0, z, dims, 1);
                let direct = decode_percentage_from_z_indexes(&flag, depth, &[z], &[]).unwrap();
                assert!(voxel.success);
                assert_eq!(voxel.value_0_1, direct, "unsigned {pos:?} z={z}");
            }
        }
        // Signed along-z (one-dimensional signed linear, single X column)
        let flag = signed_flag(LINEAR);
        let id = flag.as_io_cortical_id(false, *b"omo", 0u16.into(), 0u8.into());
        for z in 0..depth {
            let voxel = decode_single_voxel(&id, 0, 0, z, dims, 1);
            let direct = decode_percentage_from_z_indexes(&flag, depth, &[z], &[]).unwrap();
            assert_eq!(voxel.value_percent, direct * 100.0, "along-z z={z}");
        }
    }

    #[test]
    fn decode_unsigned_linear_known_values() {
        let flag = unsigned_flag(LINEAR);
        assert_eq!(
            decode_percentage_from_z_indexes(&flag, 10, &[0], &[]),
            Ok(1.0)
        );
        assert_eq!(
            decode_percentage_from_z_indexes(&flag, 10, &[9], &[]),
            Ok(0.0)
        );
        // mean of z 3 and 5 is 4 -> 1 - 4/9
        let v = decode_percentage_from_z_indexes(&flag, 10, &[3, 5], &[]).unwrap();
        assert!((v - (1.0 - 4.0 / 9.0)).abs() < 1e-6);
    }

    #[test]
    fn decode_signed_variants_known_values() {
        // along-z: z 0 -> +1, z depth-1 -> -1
        let along = signed_flag(LINEAR);
        assert_eq!(
            decode_percentage_from_z_indexes(&along, 9, &[0], &[]),
            Ok(1.0)
        );
        assert_eq!(
            decode_percentage_from_z_indexes(&along, 9, &[8], &[]),
            Ok(-1.0)
        );
        // two-lobe linear: positive z 0 -> +1, negative z 0 -> -1, equal lobes cancel
        let lobes = signed_2d_flag(LINEAR);
        assert_eq!(
            decode_percentage_from_z_indexes(&lobes, 10, &[0], &[]),
            Ok(1.0)
        );
        assert_eq!(
            decode_percentage_from_z_indexes(&lobes, 10, &[], &[0]),
            Ok(-1.0)
        );
        assert_eq!(
            decode_percentage_from_z_indexes(&lobes, 10, &[9], &[9]),
            Ok(0.0)
        );
        // fractional signed: +0.5 - 0.25
        let frac = signed_flag(FRACTIONAL);
        assert_eq!(
            decode_percentage_from_z_indexes(&frac, 8, &[0], &[1]),
            Ok(0.25)
        );
    }

    #[test]
    fn decode_rejects_bad_inputs() {
        let unsigned = unsigned_flag(LINEAR);
        assert_eq!(
            decode_percentage_from_z_indexes(&unsigned, 0, &[0], &[]),
            Err(PercentageCodingError::ZeroDepth)
        );
        assert_eq!(
            decode_percentage_from_z_indexes(&unsigned, 4, &[4], &[]),
            Err(PercentageCodingError::ZIndexOutOfRange { z: 4, depth: 4 })
        );
        assert_eq!(
            decode_percentage_from_z_indexes(&unsigned, 4, &[], &[]),
            Err(PercentageCodingError::NoActiveZIndex)
        );
        // negative list is only valid on two-lobe variants
        assert!(matches!(
            decode_percentage_from_z_indexes(&unsigned, 4, &[0], &[1]),
            Err(PercentageCodingError::NegativeLobeNotApplicable(_))
        ));
        assert!(matches!(
            decode_percentage_from_z_indexes(&signed_flag(LINEAR), 4, &[0], &[1]),
            Err(PercentageCodingError::NegativeLobeNotApplicable(_))
        ));
    }

    #[test]
    fn encode_selects_along_z_or_two_lobe_by_flag() {
        // Same one-dimensional signed linear flag family: scalar -> along-z (one column),
        // 2D -> two-lobe (positive and negative lists).
        let along = encode_percentage_to_z_indexes(&signed_flag(LINEAR), 9, -1.0).unwrap();
        assert_eq!(along.positive, vec![8]);
        assert!(along.negative.is_empty());
        let lobes = encode_percentage_to_z_indexes(&signed_2d_flag(LINEAR), 10, -1.0).unwrap();
        assert!(lobes.positive.is_empty());
        assert_eq!(lobes.negative, vec![0]);
    }

    #[test]
    fn encode_rejects_out_of_range_and_zero_depth() {
        for bad in [f32::NAN, f32::INFINITY, 1.5, -0.1] {
            assert!(matches!(
                encode_percentage_to_z_indexes(&unsigned_flag(LINEAR), 8, bad),
                Err(PercentageCodingError::ValueOutOfRange { .. })
            ));
        }
        assert!(encode_percentage_to_z_indexes(&signed_flag(LINEAR), 8, -1.1).is_err());
        assert_eq!(
            encode_percentage_to_z_indexes(&unsigned_flag(LINEAR), 0, 0.5),
            Err(PercentageCodingError::ZeroDepth)
        );
    }

    #[test]
    fn round_trip_is_within_one_bin_not_exact() {
        // Documented encoder/decoder asymmetry: encode places floor((1 - v) * depth), decode
        // reads 1 - z / (depth - 1), so the round trip is within one bin and not exact.
        let depth = 10u32;
        let bin = 1.0 / (depth - 1) as f32;
        let mut inexact = 0;
        for i in 0..=100 {
            let v = i as f32 / 100.0;
            let flag = unsigned_flag(LINEAR);
            let z = encode_percentage_to_z_indexes(&flag, depth, v).unwrap();
            let back =
                decode_percentage_from_z_indexes(&flag, depth, &z.positive, &z.negative).unwrap();
            assert!((back - v).abs() <= bin + 1e-6, "v={v} back={back}");
            if (back - v).abs() > 1e-6 {
                inexact += 1;
            }
        }
        assert!(
            inexact > 0,
            "the asymmetry means some values do not round trip exactly"
        );
        // Signed along-z and fractional: within one bin / 2^-depth respectively.
        for i in -10..=10 {
            let v = i as f32 / 10.0;
            let flag = signed_flag(LINEAR);
            let z = encode_percentage_to_z_indexes(&flag, depth, v).unwrap();
            let back =
                decode_percentage_from_z_indexes(&flag, depth, &z.positive, &z.negative).unwrap();
            assert!(
                (back - v).abs() <= 2.0 * bin + 1e-6,
                "along-z v={v} back={back}"
            );
            let flag = signed_flag(FRACTIONAL);
            let z = encode_percentage_to_z_indexes(&flag, depth, v).unwrap();
            let back =
                decode_percentage_from_z_indexes(&flag, depth, &z.positive, &z.negative).unwrap();
            assert!(
                (back - v).abs() <= 0.5f32.powi(depth as i32) + 1e-6,
                "fractional v={v} back={back}"
            );
        }
    }

    #[test]
    fn target_to_z_bin_matches_python_value_to_z_bin() {
        // Saved stance mappings must not shift: the Rust inverse equals the Python tool's.
        let flag = unsigned_flag(LINEAR);
        for (depth, expected) in PYTHON_VALUE_TO_Z_BIN {
            for (value, want) in INVERSE_VALUES.iter().zip(expected) {
                assert_eq!(
                    target_to_z_bin(&flag, depth, *value),
                    Ok(want),
                    "depth={depth} value={value}"
                );
            }
        }
    }

    #[test]
    fn target_to_z_bin_is_nearest_decode_for_linear_variants() {
        // Cross-check the closed form against the real decoder: no other bin decodes closer.
        for flag in [unsigned_flag(LINEAR), signed_flag(LINEAR)] {
            let (lo, hi) = if PercentageCodingVariant::from_io_flag(&flag)
                .unwrap()
                .is_signed()
            {
                (-1.0f64, 1.0f64)
            } else {
                (0.0, 1.0)
            };
            for depth in 2..=17u32 {
                for step in 0..=40 {
                    let target = lo + (hi - lo) * step as f64 / 40.0;
                    let chosen = target_to_z_bin(&flag, depth, target).unwrap();
                    let dist = |z: u32| {
                        let v = decode_percentage_from_z_indexes(&flag, depth, &[z], &[]).unwrap();
                        (v as f64 - target).abs()
                    };
                    let best = (0..depth).map(dist).fold(f64::INFINITY, f64::min);
                    assert!(
                        dist(chosen) <= best + 1e-6,
                        "depth={depth} target={target} chose z={chosen}"
                    );
                }
            }
        }
    }

    #[test]
    fn target_to_z_bin_decodes_back_within_half_a_bin() {
        let depth = 10u32;
        let half_bin = 0.5 / (depth - 1) as f64;
        for flag in [unsigned_flag(LINEAR), signed_flag(LINEAR)] {
            let signed = PercentageCodingVariant::from_io_flag(&flag)
                .unwrap()
                .is_signed();
            let bin = if signed { 2.0 * half_bin } else { half_bin };
            for step in 0..=20 {
                let target = if signed {
                    -1.0 + step as f64 / 10.0
                } else {
                    step as f64 / 20.0
                };
                let z = target_to_z_bin(&flag, depth, target).unwrap();
                let back = decode_percentage_from_z_indexes(&flag, depth, &[z], &[]).unwrap();
                assert!(
                    (back as f64 - target).abs() <= bin + 1e-6,
                    "target={target}"
                );
            }
        }
    }

    #[test]
    fn target_to_z_bin_endpoints_and_single_bin_depth() {
        let flag = unsigned_flag(LINEAR);
        assert_eq!(target_to_z_bin(&flag, 10, 1.0), Ok(0));
        assert_eq!(target_to_z_bin(&flag, 10, 0.0), Ok(9));
        assert_eq!(target_to_z_bin(&flag, 1, 0.3), Ok(0));
        let along = signed_flag(LINEAR);
        assert_eq!(target_to_z_bin(&along, 9, 1.0), Ok(0));
        assert_eq!(target_to_z_bin(&along, 9, -1.0), Ok(8));
        assert_eq!(target_to_z_bin(&along, 9, 0.0), Ok(4));
        assert_eq!(target_to_z_bin(&along, 1, 0.5), Ok(0));
    }

    #[test]
    fn target_to_z_bin_unsigned_fractional_is_nearest_single_bin() {
        let flag = unsigned_flag(FRACTIONAL);
        assert_eq!(target_to_z_bin(&flag, 8, 0.5), Ok(0));
        assert_eq!(target_to_z_bin(&flag, 8, 0.25), Ok(1));
        assert_eq!(target_to_z_bin(&flag, 8, 0.0), Ok(7));
        // 0.4 is nearer 0.5 (z 0) than 0.25 (z 1)
        assert_eq!(target_to_z_bin(&flag, 8, 0.4), Ok(0));
    }

    #[test]
    fn target_to_z_bin_explicit_errors() {
        // no single-bin inverse for the two-lobe variants
        for flag in [
            signed_2d_flag(LINEAR),
            signed_flag(FRACTIONAL),
            signed_2d_flag(FRACTIONAL),
        ] {
            assert!(matches!(
                target_to_z_bin(&flag, 8, 0.5),
                Err(PercentageCodingError::NoSingleBinInverse { .. })
            ));
        }
        // targets are not clamped
        let flag = unsigned_flag(LINEAR);
        for bad in [-0.01, 1.01, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                target_to_z_bin(&flag, 8, bad),
                Err(PercentageCodingError::ValueOutOfRange { .. })
            ));
        }
        assert_eq!(
            target_to_z_bin(&flag, 0, 0.5),
            Err(PercentageCodingError::ZeroDepth)
        );
        assert!(target_to_z_bin(&IOCorticalAreaConfigurationFlag::Boolean, 8, 0.5).is_err());
    }

    //endregion
}
