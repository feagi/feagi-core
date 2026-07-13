use crate::cortical_area::CorticalID;
use crate::cortical_unit::{CorticalSubUnitIndex, CorticalUnitIndex};
use crate::feagi_genome_context_error::{FeagiCorticalConfigurationFlagErrKey, FeagiGenomeContextError};
use serde::{Deserialize, Serialize};
use std::fmt;

pub type IOCorticalAreaConfigurationFlagBitmask = u16; // 16 Total bits

/// Define the indexes of various bit flags
mod bit_indexes {
    // Bits 0-7 -> Enum variant discriminant
    pub const FRAME_CHANGE_HANDLING: usize = 8;
    pub const PERCENTAGE_NEURON_POSITIONING: usize = 9;
    // Bits 10-12 -> PoseSchema (3 bits, used only by PoseEstimation variant)
    pub const POSE_SCHEMA_START: usize = 10;
    pub const POSE_SCHEMA_MASK: u16 = 0b111; // 3 bits
                                             // Bits 13-15 -> RESERVED
}

/// Different types of Input/Output cortical_area areas exist, and have their own nested configurations. This enum defines that
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum IOCorticalAreaConfigurationFlag {
    Boolean,
    Percentage(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    CartesianPlane(FrameChangeHandling),
    Misc(FrameChangeHandling),
    PoseEstimation(FrameChangeHandling, PoseSchema),
}

impl IOCorticalAreaConfigurationFlag {
    pub const fn try_from_data_type_configuration_flag(value: IOCorticalAreaConfigurationFlagBitmask) -> Result<Self, FeagiGenomeContextError> {
        let variant = value & 0xFF; // Bits 0-7
        let frame_handling = (value >> bit_indexes::FRAME_CHANGE_HANDLING) & 0x01;
        let positioning = (value >> bit_indexes::PERCENTAGE_NEURON_POSITIONING) & 0x01;

        let frame_handling_enum = match frame_handling {
            0 => FrameChangeHandling::Absolute,
            1 => FrameChangeHandling::Incremental,
            _ => {
                return Err(FeagiGenomeContextError::const_from_configuration_flag(
                    FeagiCorticalConfigurationFlagErrKey::new("invalid frame handling bit in IO cortical_area configuration flag"),
                ));
            }
        };

        let positioning_enum = match positioning {
            0 => PercentageNeuronPositioning::Linear,
            1 => PercentageNeuronPositioning::Fractional,
            _ => {
                return Err(FeagiGenomeContextError::const_from_configuration_flag(
                    FeagiCorticalConfigurationFlagErrKey::new("invalid neuron positioning bit in IO cortical_area configuration flag"),
                ));
            }
        };

        match variant {
            0 => Ok(IOCorticalAreaConfigurationFlag::Boolean),
            1 => Ok(IOCorticalAreaConfigurationFlag::Percentage(frame_handling_enum, positioning_enum)),
            2 => Ok(IOCorticalAreaConfigurationFlag::Percentage2D(frame_handling_enum, positioning_enum)),
            3 => Ok(IOCorticalAreaConfigurationFlag::Percentage3D(frame_handling_enum, positioning_enum)),
            4 => Ok(IOCorticalAreaConfigurationFlag::Percentage4D(frame_handling_enum, positioning_enum)),
            5 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage(frame_handling_enum, positioning_enum)),
            6 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage2D(frame_handling_enum, positioning_enum)),
            7 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame_handling_enum, positioning_enum)),
            8 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage4D(frame_handling_enum, positioning_enum)),
            9 => {
                if positioning != 0 {
                    return Err(FeagiGenomeContextError::const_from_configuration_flag(
                        FeagiCorticalConfigurationFlagErrKey::new("IO CartesianPlane configuration does not allow positioning"),
                    ));
                }
                Ok(IOCorticalAreaConfigurationFlag::CartesianPlane(frame_handling_enum))
            }
            10 => {
                if positioning != 0 {
                    return Err(FeagiGenomeContextError::const_from_configuration_flag(
                        FeagiCorticalConfigurationFlagErrKey::new("IO CartesianPlane configuration does not allow positioning"),
                    ));
                }
                Ok(IOCorticalAreaConfigurationFlag::Misc(frame_handling_enum))
            }
            11 => {
                if positioning != 0 {
                    return Err(FeagiGenomeContextError::const_from_configuration_flag(
                        FeagiCorticalConfigurationFlagErrKey::new("IO CartesianPlane configuration does not allow positioning"),
                    ));
                }
                let pose_schema_bits = (value >> bit_indexes::POSE_SCHEMA_START) & bit_indexes::POSE_SCHEMA_MASK;
                let pose_schema = match pose_schema_bits {
                    0 => PoseSchema::HumanBody,
                    1 => PoseSchema::HumanHand,
                    2 => PoseSchema::HumanFace,
                    3 => PoseSchema::Quadruped,
                    4 => PoseSchema::Avian,
                    5 => PoseSchema::Arthropod,
                    6 => PoseSchema::Object6DoF,
                    7 => PoseSchema::Custom,
                    _ => {
                        return Err(FeagiGenomeContextError::const_from_configuration_flag(
                            FeagiCorticalConfigurationFlagErrKey::new("IO CartesianPlane configuration does not allow positioning"),
                        ));
                    }
                };
                Ok(IOCorticalAreaConfigurationFlag::PoseEstimation(frame_handling_enum, pose_schema))
            }
            _ => Err(FeagiGenomeContextError::const_from_configuration_flag(
                FeagiCorticalConfigurationFlagErrKey::new("invalid IO cortical_area configuration variant"),
            )),
        }
    }

    pub const fn to_data_type_configuration_flag(&self) -> IOCorticalAreaConfigurationFlagBitmask {
        let (variant, frame_handling, positioning, pose_schema) = match self {
            IOCorticalAreaConfigurationFlag::Boolean => (0u16, None, None, None),
            IOCorticalAreaConfigurationFlag::Percentage(f, p) => (1u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::Percentage2D(f, p) => (2u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::Percentage3D(f, p) => (3u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::Percentage4D(f, p) => (4u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::SignedPercentage(f, p) => (5u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::SignedPercentage2D(f, p) => (6u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(f, p) => (7u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::SignedPercentage4D(f, p) => (8u16, Some(*f), Some(*p), None),
            IOCorticalAreaConfigurationFlag::CartesianPlane(f) => (9u16, Some(*f), None, None),
            IOCorticalAreaConfigurationFlag::Misc(f) => (10u16, Some(*f), None, None),
            IOCorticalAreaConfigurationFlag::PoseEstimation(f, s) => (11u16, Some(*f), None, Some(*s)),
        };

        let frame_bits = match frame_handling {
            Some(FrameChangeHandling::Absolute) => 0u16,
            Some(FrameChangeHandling::Incremental) => 1u16,
            None => 0u16,
        };

        let positioning_bits = match positioning {
            Some(PercentageNeuronPositioning::Linear) => 0u16,
            Some(PercentageNeuronPositioning::Fractional) => 1u16,
            None => 0u16,
        };

        let pose_schema_bits = match pose_schema {
            Some(s) => s.to_bits(),
            None => 0u16,
        };

        variant
            | (frame_bits << bit_indexes::FRAME_CHANGE_HANDLING)
            | (positioning_bits << bit_indexes::PERCENTAGE_NEURON_POSITIONING)
            | (pose_schema_bits << bit_indexes::POSE_SCHEMA_START)
    }

    pub const fn as_io_cortical_id(
        &self,
        is_input: bool,
        cortical_unit_identifier: [u8; 3],
        cortical_unit_index: CorticalUnitIndex,
        cortical_sub_unit_index: CorticalSubUnitIndex,
    ) -> CorticalID {
        let data_type_configuration: IOCorticalAreaConfigurationFlagBitmask = self.to_data_type_configuration_flag();
        let data_type_configuration_bytes: [u8; 2] = data_type_configuration.to_le_bytes();

        let cortical_id_bytes: [u8; CorticalID::NUMBER_OF_BYTES] = [
            if is_input { b'i' } else { b'o' },
            cortical_unit_identifier[0],
            cortical_unit_identifier[1],
            cortical_unit_identifier[2],
            data_type_configuration_bytes[0],
            data_type_configuration_bytes[1],
            cortical_sub_unit_index.const_deref(),
            cortical_unit_index.const_deref(),
        ];

        CorticalID { bytes: cortical_id_bytes }
    }
}

impl From<&IOCorticalAreaConfigurationFlag> for IOCorticalAreaConfigurationFlagBitmask {
    fn from(data_type: &IOCorticalAreaConfigurationFlag) -> Self {
        data_type.to_data_type_configuration_flag()
    }
}

impl From<IOCorticalAreaConfigurationFlag> for IOCorticalAreaConfigurationFlagBitmask {
    fn from(data_type: IOCorticalAreaConfigurationFlag) -> Self {
        (&data_type).into()
    }
}

impl TryFrom<IOCorticalAreaConfigurationFlagBitmask> for IOCorticalAreaConfigurationFlag {
    type Error = FeagiGenomeContextError;

    fn try_from(value: IOCorticalAreaConfigurationFlagBitmask) -> Result<Self, Self::Error> {
        IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(value)
    }
}

impl fmt::Display for IOCorticalAreaConfigurationFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOCorticalAreaConfigurationFlag::Boolean => write!(f, "Boolean()"),
            IOCorticalAreaConfigurationFlag::Percentage(frame, percentage) => {
                write!(f, "Percentage({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::Percentage2D(frame, percentage) => {
                write!(f, "Percentage2D({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::Percentage3D(frame, percentage) => {
                write!(f, "Percentage3D({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::Percentage4D(frame, percentage) => {
                write!(f, "Percentage4D({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage(frame, percentage) => {
                write!(f, "SignedPercentage({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage2D(frame, percentage) => {
                write!(f, "SignedPercentage2D({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, percentage) => {
                write!(f, "SignedPercentage3D({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage4D(frame, percentage) => {
                write!(f, "SignedPercentage4D({}, {})", frame, percentage)
            }
            IOCorticalAreaConfigurationFlag::CartesianPlane(frame) => {
                write!(f, "CartesianPlane({})", frame)
            }
            IOCorticalAreaConfigurationFlag::Misc(frame) => write!(f, "Misc({})", frame),
            IOCorticalAreaConfigurationFlag::PoseEstimation(frame, schema) => {
                write!(f, "PoseEstimation({}, {})", frame, schema)
            }
        }
    }
}

//region SubEnums
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PercentageNeuronPositioning {
    Linear,
    #[default]
    Fractional,
}

impl PercentageNeuronPositioning {
    pub fn try_from_serde_map(map: &serde_json::Map<String, serde_json::Value>) -> Result<PercentageNeuronPositioning, FeagiGenomeContextError> {
        let val = map
            .get("percentage_neuron_positioning")
            .ok_or(FeagiGenomeContextError::const_from_configuration_flag(
                FeagiCorticalConfigurationFlagErrKey::new("missing or invalid percentage_neuron_positioning in serde map"),
            ))?;
        let output: PercentageNeuronPositioning = serde_json::from_value(val.clone()).map_err(|_| {
            FeagiGenomeContextError::const_from_configuration_flag(FeagiCorticalConfigurationFlagErrKey::new(
                "missing or invalid percentage_neuron_positioning in serde map",
            ))
        })?;
        Ok(output)
    }
}

impl fmt::Display for PercentageNeuronPositioning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PercentageNeuronPositioning::Linear => write!(f, "Linear"),
            PercentageNeuronPositioning::Fractional => write!(f, "Fractional"),
        }
    }
}

/// Returns the area configuration flag for a SpatialPointer cortical_area area.
///
/// SpatialPointer encodes its decode mechanism in the area's data type so the genome is
/// self-describing for consumers:
/// - `Absolute` decodes an unsigned position centroid, so the area is `Percentage3D`
///   (each axis in `[0, 1]`).
/// - `Incremental` decodes a signed motion vector, so the area is `SignedPercentage3D`
///   (each axis in `[-1, 1]`, `0` meaning no motion).
///
/// Signedness is therefore fully determined by the frame-change mode; there is no valid
/// "absolute + signed" or "incremental + unsigned" combination.
pub const fn spatial_pointer_io_flag(
    frame_change_handling: FrameChangeHandling,
    percentage_neuron_positioning: PercentageNeuronPositioning,
) -> IOCorticalAreaConfigurationFlag {
    match frame_change_handling {
        FrameChangeHandling::Absolute => IOCorticalAreaConfigurationFlag::Percentage3D(frame_change_handling, percentage_neuron_positioning),
        FrameChangeHandling::Incremental => IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame_change_handling, percentage_neuron_positioning),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FrameChangeHandling {
    #[default]
    Absolute,
    Incremental,
}

impl FrameChangeHandling {
    pub fn try_from_serde_map(map: &serde_json::Map<String, serde_json::Value>) -> Result<FrameChangeHandling, FeagiGenomeContextError> {
        let val = map
            .get("frame_change_handling")
            .ok_or(FeagiGenomeContextError::const_from_configuration_flag(
                FeagiCorticalConfigurationFlagErrKey::new("missing or invalid frame_change_handling in serde map"),
            ))?;
        let output: FrameChangeHandling = serde_json::from_value(val.clone()).map_err(|_| {
            FeagiGenomeContextError::const_from_configuration_flag(FeagiCorticalConfigurationFlagErrKey::new(
                "missing or invalid frame_change_handling in serde map",
            ))
        })?;
        Ok(output)
    }
}

impl fmt::Display for FrameChangeHandling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameChangeHandling::Absolute => write!(f, "Absolute"),
            FrameChangeHandling::Incremental => write!(f, "Incremental"),
        }
    }
}

/// Pose estimation schema encoded in the cortical_area ID (bits 10-12, 3 bits = 8 values).
/// The super class identifies the joint topology; combined with Z depth it uniquely
/// determines the sub-class (e.g. HumanBody + Z=17 = COCO-17).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PoseSchema {
    #[default]
    HumanBody,
    HumanHand,
    HumanFace,
    Quadruped,
    Avian,
    Arthropod,
    Object6DoF,
    Custom,
}

impl PoseSchema {
    pub const fn to_bits(self) -> u16 {
        match self {
            PoseSchema::HumanBody => 0,
            PoseSchema::HumanHand => 1,
            PoseSchema::HumanFace => 2,
            PoseSchema::Quadruped => 3,
            PoseSchema::Avian => 4,
            PoseSchema::Arthropod => 5,
            PoseSchema::Object6DoF => 6,
            PoseSchema::Custom => 7,
        }
    }

    pub const fn try_from_bits(bits: u16) -> Result<Self, FeagiGenomeContextError> {
        match bits {
            0 => Ok(PoseSchema::HumanBody),
            1 => Ok(PoseSchema::HumanHand),
            2 => Ok(PoseSchema::HumanFace),
            3 => Ok(PoseSchema::Quadruped),
            4 => Ok(PoseSchema::Avian),
            5 => Ok(PoseSchema::Arthropod),
            6 => Ok(PoseSchema::Object6DoF),
            7 => Ok(PoseSchema::Custom),
            _ => Err(FeagiGenomeContextError::const_from_configuration_flag(
                FeagiCorticalConfigurationFlagErrKey::new("invalid PoseSchema bits in IO cortical_area configuration flag"),
            )),
        }
    }

    pub fn try_from_serde_map(map: &serde_json::Map<String, serde_json::Value>) -> Result<PoseSchema, FeagiGenomeContextError> {
        let val = map.get("pose_schema").ok_or(FeagiGenomeContextError::const_from_configuration_flag(
            FeagiCorticalConfigurationFlagErrKey::new("missing or invalid pose_schema in serde map"),
        ))?;
        let output: PoseSchema = serde_json::from_value(val.clone()).map_err(|_| {
            FeagiGenomeContextError::const_from_configuration_flag(FeagiCorticalConfigurationFlagErrKey::new(
                "missing or invalid pose_schema in serde map",
            ))
        })?;
        Ok(output)
    }
}

impl fmt::Display for PoseSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoseSchema::HumanBody => write!(f, "HumanBody"),
            PoseSchema::HumanHand => write!(f, "HumanHand"),
            PoseSchema::HumanFace => write!(f, "HumanFace"),
            PoseSchema::Quadruped => write!(f, "Quadruped"),
            PoseSchema::Avian => write!(f, "Avian"),
            PoseSchema::Arthropod => write!(f, "Arthropod"),
            PoseSchema::Object6DoF => write!(f, "Object6DoF"),
            PoseSchema::Custom => write!(f, "Custom"),
        }
    }
}
//endregion
