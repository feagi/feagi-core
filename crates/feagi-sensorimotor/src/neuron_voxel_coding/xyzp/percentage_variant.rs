//! The single selector that maps an IO configuration to a percentage coding variant, and the
//! one place that dispatches a variant to the shared encode / decode functions.
//!
//! The batch encoder (`encoders/percentage_encoder.rs`), the batch decoder
//! (`decoders/percentage_decoder.rs`), the single-voxel decoder (`single_voxel_decode.rs`) and
//! the public coder functions all choose their coding through [`PercentageCodingVariant`], so a
//! given IO configuration flag can never select different codings in different paths.
//!
//! Encoder/decoder asymmetry (documented, not a bug): the linear encoder places a value at
//! `floor((1 - v) * depth)` clamped to `depth - 1`, while the unsigned linear decoder reads a
//! z index as `1 - z / (depth - 1)`. A round trip therefore returns the input only within
//! about one bin (`1 / (depth - 1)`), never exactly. [`PercentageCodingVariant`] users that
//! need the z bin for a wanted decoded value use `target_to_z_bin`, which inverts the decoder.

use super::coder_shared_functions::{
    decode_signed_percentage_from_fractional_exponential_neurons,
    decode_signed_percentage_from_linear_neurons,
    decode_signed_percentage_from_linear_neurons_along_z,
    decode_unsigned_percentage_from_fractional_exponential_neurons,
    decode_unsigned_percentage_from_linear_neurons,
    encode_signed_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_signed_percentage_to_linear_neuron_z_index,
    encode_signed_percentage_to_linear_neuron_z_index_along_z,
    encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes,
    encode_unsigned_percentage_to_linear_neuron_z_index,
};
use crate::data_types::{Percentage, SignedPercentage};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
use std::fmt;

/// Which z-axis coding a percentage channel uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PercentageCodingVariant {
    /// Unsigned, one active z bin per value; z = 0 is the maximum (1.0).
    UnsignedLinear,
    /// Unsigned, value is a sum of `0.5^(z + 1)` over the active z bins.
    UnsignedFractional,
    /// Signed, one X column per value; z = 0 is +1.0 and z = depth - 1 is -1.0.
    SignedLinearAlongZ,
    /// Signed, two X columns per value (positive lobe, negative lobe), each lobe linear.
    SignedLinearTwoLobe,
    /// Signed, two X columns per value (positive lobe, negative lobe), each lobe fractional.
    SignedFractional,
}

impl PercentageCodingVariant {
    /// Select the variant from the coding properties of a percentage channel.
    ///
    /// * `is_signed` - signed (-1..1) or unsigned (0..1) value.
    /// * `is_scalar` - true for a one-dimensional percentage, false for 2D, 3D or 4D.
    /// * `positioning` - linear or fractional neuron positioning along z.
    ///
    /// A signed one-dimensional linear percentage uses the single-column along-z layout; every
    /// other signed linear percentage uses two lobes.
    pub fn from_parts(
        is_signed: bool,
        is_scalar: bool,
        positioning: PercentageNeuronPositioning,
    ) -> Self {
        match (is_signed, positioning) {
            (false, PercentageNeuronPositioning::Linear) => Self::UnsignedLinear,
            (false, PercentageNeuronPositioning::Fractional) => Self::UnsignedFractional,
            (true, PercentageNeuronPositioning::Linear) if is_scalar => Self::SignedLinearAlongZ,
            (true, PercentageNeuronPositioning::Linear) => Self::SignedLinearTwoLobe,
            (true, PercentageNeuronPositioning::Fractional) => Self::SignedFractional,
        }
    }

    /// Select the variant from an IO configuration flag.
    ///
    /// Only the `Percentage*` and `SignedPercentage*` flags carry a percentage coding; any
    /// other flag is an error.
    pub fn from_io_flag(
        flag: &IOCorticalAreaConfigurationFlag,
    ) -> Result<Self, PercentageCodingError> {
        use IOCorticalAreaConfigurationFlag as Flag;
        match flag {
            Flag::Percentage(_, pos) => Ok(Self::from_parts(false, true, *pos)),
            Flag::Percentage2D(_, pos)
            | Flag::Percentage3D(_, pos)
            | Flag::Percentage4D(_, pos) => Ok(Self::from_parts(false, false, *pos)),
            Flag::SignedPercentage(_, pos) => Ok(Self::from_parts(true, true, *pos)),
            Flag::SignedPercentage2D(_, pos)
            | Flag::SignedPercentage3D(_, pos)
            | Flag::SignedPercentage4D(_, pos) => Ok(Self::from_parts(true, false, *pos)),
            other => Err(PercentageCodingError::NotAPercentageFlag(format!(
                "{other:?}"
            ))),
        }
    }

    /// True for the variants that decode to -1..1.
    pub fn is_signed(self) -> bool {
        !matches!(self, Self::UnsignedLinear | Self::UnsignedFractional)
    }

    /// True for the variants that carry a separate positive and negative lobe.
    pub fn uses_two_lobes(self) -> bool {
        matches!(self, Self::SignedLinearTwoLobe | Self::SignedFractional)
    }

    /// Neuron positioning along z of this variant.
    pub fn positioning(self) -> PercentageNeuronPositioning {
        match self {
            Self::UnsignedFractional | Self::SignedFractional => {
                PercentageNeuronPositioning::Fractional
            }
            _ => PercentageNeuronPositioning::Linear,
        }
    }

    /// Decode an unsigned value from its active z indexes into `target`.
    ///
    /// Panics (debug) if the variant is signed; callers select by [`Self::is_signed`].
    pub(crate) fn decode_unsigned_into(self, z: &[u32], depth: u32, target: &mut Percentage) {
        match self {
            Self::UnsignedLinear => {
                decode_unsigned_percentage_from_linear_neurons(z, depth, target)
            }
            Self::UnsignedFractional => {
                decode_unsigned_percentage_from_fractional_exponential_neurons(z, target)
            }
            _ => debug_assert!(false, "decode_unsigned_into called on signed {self:?}"),
        }
    }

    /// Decode a signed value into `target`. `z_negative` is read only by the two-lobe variants;
    /// the along-z variant reads `z_positive` as its single column.
    pub(crate) fn decode_signed_into(
        self,
        z_positive: &[u32],
        z_negative: &[u32],
        depth: u32,
        target: &mut SignedPercentage,
    ) {
        match self {
            Self::SignedLinearAlongZ => {
                decode_signed_percentage_from_linear_neurons_along_z(z_positive, depth, target)
            }
            Self::SignedLinearTwoLobe => {
                decode_signed_percentage_from_linear_neurons(z_positive, z_negative, depth, target)
            }
            Self::SignedFractional => decode_signed_percentage_from_fractional_exponential_neurons(
                z_positive, z_negative, target,
            ),
            _ => debug_assert!(false, "decode_signed_into called on unsigned {self:?}"),
        }
    }

    /// Encode an unsigned value into the active z indexes `z` (cleared first).
    pub(crate) fn encode_unsigned_into(self, value: &Percentage, depth: u32, z: &mut Vec<u32>) {
        match self {
            Self::UnsignedLinear => {
                encode_unsigned_percentage_to_linear_neuron_z_index(value, depth as f32, z)
            }
            Self::UnsignedFractional => {
                encode_unsigned_percentage_to_fractional_exponential_neuron_z_indexes(
                    value, depth, z,
                )
            }
            _ => debug_assert!(false, "encode_unsigned_into called on signed {self:?}"),
        }
    }

    /// Encode a signed value into the active z indexes (both cleared first). The along-z
    /// variant writes `z_positive` only and clears `z_negative`.
    pub(crate) fn encode_signed_into(
        self,
        value: &SignedPercentage,
        depth: u32,
        z_positive: &mut Vec<u32>,
        z_negative: &mut Vec<u32>,
    ) {
        match self {
            Self::SignedLinearAlongZ => {
                encode_signed_percentage_to_linear_neuron_z_index_along_z(
                    value,
                    depth as f32,
                    z_positive,
                );
                z_negative.clear();
            }
            Self::SignedLinearTwoLobe => encode_signed_percentage_to_linear_neuron_z_index(
                value,
                depth as f32,
                z_positive,
                z_negative,
            ),
            Self::SignedFractional => {
                encode_signed_percentage_to_fractional_exponential_neuron_z_indexes(
                    value, depth, z_positive, z_negative,
                )
            }
            _ => debug_assert!(false, "encode_signed_into called on unsigned {self:?}"),
        }
    }
}

/// Error from the public percentage coder functions.
#[derive(Debug, Clone, PartialEq)]
pub enum PercentageCodingError {
    /// The IO configuration flag is not a percentage flag.
    NotAPercentageFlag(String),
    /// Depth (number of z bins) must be at least 1.
    ZeroDepth,
    /// An active z index lies outside `0..depth`.
    ZIndexOutOfRange { z: u32, depth: u32 },
    /// A negative-lobe list was given to a variant that has no negative lobe.
    NegativeLobeNotApplicable(PercentageCodingVariant),
    /// An unsigned linear value needs at least one active z index.
    NoActiveZIndex,
    /// The value or target is not finite or lies outside the variant's range.
    ValueOutOfRange { value: f64, min: f64, max: f64 },
    /// The variant has no single-z-bin inverse; the reason names why.
    NoSingleBinInverse {
        variant: PercentageCodingVariant,
        reason: &'static str,
    },
}

impl fmt::Display for PercentageCodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAPercentageFlag(flag) => {
                write!(f, "IO configuration flag is not a percentage flag: {flag}")
            }
            Self::ZeroDepth => write!(f, "depth must be at least 1"),
            Self::ZIndexOutOfRange { z, depth } => {
                write!(f, "active z index {z} is outside 0..{depth}")
            }
            Self::NegativeLobeNotApplicable(variant) => {
                write!(
                    f,
                    "{variant:?} has no negative lobe; the negative list must be empty"
                )
            }
            Self::NoActiveZIndex => write!(f, "unsigned linear decode needs an active z index"),
            Self::ValueOutOfRange { value, min, max } => {
                write!(f, "value {value} is not finite or outside [{min}, {max}]")
            }
            Self::NoSingleBinInverse { variant, reason } => {
                write!(f, "{variant:?} has no single-z-bin inverse: {reason}")
            }
        }
    }
}

impl std::error::Error for PercentageCodingError {}
