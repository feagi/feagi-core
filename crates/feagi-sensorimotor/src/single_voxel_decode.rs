//! Single-voxel decoding for hover/inspection UIs.
//!
//! Uses the same decoding logic as feagi-sensorimotor's batch decoders so that
//! what Brain Visualizer displays matches what a robot/controller would process.

use crate::data_types::{Percentage, SignedPercentage};
use crate::neuron_voxel_coding::xyzp::coder_shared_functions::{
    decode_signed_percentage_from_fractional_exponential_neurons,
    decode_signed_percentage_from_linear_neurons,
    decode_signed_percentage_from_linear_neurons_along_z,
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
};
use feagi_genomic_context::cortical_area::io_cortical_area_configuration_flag::{
    IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
use feagi_genomic_context::cortical_area::CorticalID;
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

/// Dimensions of one channel's voxel block (from cortical_area area genome).
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

/// Decode a single voxel using encoding from cortical_area area (genome).
///
/// Used when cortical_area ID has no binary config (e.g. legacy ASCII IDs). Caller provides
/// encoding from the cortical_area area. Uses the same formulas as the primary decode path.
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

/// Decode a single voxel (x, y, z) using cortical_area ID and channel dimensions.
///
/// Uses the exact same formulas as feagi-sensorimotor's batch decoders so that
/// Brain Visualizer hover display matches what a robot/controller would process.
///
/// # Arguments
/// * `cortical_id` - The cortical_area area ID (base64 or parsed)
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

    match positioning {
        PercentageNeuronPositioning::Linear => {
            decode_unsigned_percentage_from_linear_neurons(&z_vec, ch_dim_z, &mut percentage);
        }
        PercentageNeuronPositioning::Fractional => {
            decode_unsigned_percentage_from_fractional_exponential_neurons(&z_vec, &mut percentage);
        }
    }

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

    match positioning {
        PercentageNeuronPositioning::Linear if ndim == 1 && ch_dim_x == 1 => {
            decode_signed_percentage_from_linear_neurons_along_z(&z_vec, ch_dim_z, &mut signed);
        }
        PercentageNeuronPositioning::Linear => {
            decode_signed_percentage_from_linear_neurons(&z_pos, &z_neg, ch_dim_z, &mut signed);
        }
        PercentageNeuronPositioning::Fractional => {
            decode_signed_percentage_from_fractional_exponential_neurons(
                &z_pos,
                &z_neg,
                &mut signed,
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_genomic_context::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, PercentageNeuronPositioning,
    };

    fn make_percentage_linear_id() -> CorticalID {
        let flag = IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Absolute,
            PercentageNeuronPositioning::Linear,
        );
        flag.as_io_cortical_id(false, *b"omo", 0.into(), 0.into())
    }

    fn make_percentage_fractional_id() -> CorticalID {
        let flag = IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Absolute,
            PercentageNeuronPositioning::Fractional,
        );
        flag.as_io_cortical_id(false, *b"omo", 0.into(), 0.into())
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
            (r.value_0_1 - 0.125).abs() < 0.01,
            "z=7 with z_max=8 -> 1.0 - 7/8 = 0.125 (min for linear)"
        );
    }

    #[test]
    fn test_exponential_percentage_z0() {
        let id = make_percentage_fractional_id();
        let dims = ChannelDimensions::new(1, 1, 8);
        let r = decode_single_voxel(&id, 0, 0, 0, dims, 1);
        assert!(r.success);
        assert!(
            (r.value_0_1 - 1.0).abs() < 0.01,
            "z=0 -> 0.5^0 = 1.0 (max for exponential)"
        );
    }
}
