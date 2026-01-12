use crate::genomic::cortical_area::descriptors::{CorticalSubUnitIndex, CorticalUnitIndex};
use crate::genomic::cortical_area::CorticalID;
use crate::FeagiDataError;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type IOCorticalAreaConfigurationFlagBitmask = u16; // 16 Total bits

// Bits 0-7 -> Enum
// Bit 8 -> FrameChangeHandling
// Bit 9 -> PercentageNeuronPositioning
// Bit 10-15 -> RESERVED

/// Different types of Input/Output cortical areas exist, and have their own nested configurations. This enum defines that
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
}

impl IOCorticalAreaConfigurationFlag {
    pub const fn try_from_data_type_configuration_flag(
        value: IOCorticalAreaConfigurationFlagBitmask,
    ) -> Result<Self, FeagiDataError> {
        let variant = value & 0xFF; // Bits 0-7
        let frame_handling = (value >> 8) & 0x01; // Bit 8
        let positioning = (value >> 9) & 0x01; // Bit 9

        let frame_handling_enum = match frame_handling {
            0 => FrameChangeHandling::Absolute,
            1 => FrameChangeHandling::Incremental,
            _ => return Err(FeagiDataError::ConstError("Invalid frame handling value")),
        };

        let positioning_enum = match positioning {
            0 => PercentageNeuronPositioning::Linear,
            1 => PercentageNeuronPositioning::Fractional,
            _ => return Err(FeagiDataError::ConstError("Invalid positioning value")),
        };

        match variant {
            0 => Ok(IOCorticalAreaConfigurationFlag::Boolean),
            1 => Ok(IOCorticalAreaConfigurationFlag::Percentage(
                frame_handling_enum,
                positioning_enum,
            )),
            2 => Ok(IOCorticalAreaConfigurationFlag::Percentage2D(
                frame_handling_enum,
                positioning_enum,
            )),
            3 => Ok(IOCorticalAreaConfigurationFlag::Percentage3D(
                frame_handling_enum,
                positioning_enum,
            )),
            4 => Ok(IOCorticalAreaConfigurationFlag::Percentage4D(
                frame_handling_enum,
                positioning_enum,
            )),
            5 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage(
                frame_handling_enum,
                positioning_enum,
            )),
            6 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage2D(
                frame_handling_enum,
                positioning_enum,
            )),
            7 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage3D(
                frame_handling_enum,
                positioning_enum,
            )),
            8 => Ok(IOCorticalAreaConfigurationFlag::SignedPercentage4D(
                frame_handling_enum,
                positioning_enum,
            )),
            9 => {
                // CartesianPlane doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::ConstError(
                        "CartesianPlane variant does not support positioning parameter",
                    ));
                }
                Ok(IOCorticalAreaConfigurationFlag::CartesianPlane(
                    frame_handling_enum,
                ))
            }
            10 => {
                // Misc doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::ConstError(
                        "Misc variant does not support positioning parameter",
                    ));
                }
                Ok(IOCorticalAreaConfigurationFlag::Misc(frame_handling_enum))
            }
            _ => Err(FeagiDataError::ConstError("Invalid variant type!")),
        }
    }

    pub const fn to_data_type_configuration_flag(&self) -> IOCorticalAreaConfigurationFlagBitmask {
        let (variant, frame_handling, positioning) = match self {
            IOCorticalAreaConfigurationFlag::Boolean => (0u16, None, None),
            IOCorticalAreaConfigurationFlag::Percentage(f, p) => (1u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::Percentage2D(f, p) => (2u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::Percentage3D(f, p) => (3u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::Percentage4D(f, p) => (4u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage(f, p) => (5u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage2D(f, p) => (6u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(f, p) => (7u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage4D(f, p) => (8u16, Some(*f), Some(*p)),
            IOCorticalAreaConfigurationFlag::CartesianPlane(f) => (9u16, Some(*f), None),
            IOCorticalAreaConfigurationFlag::Misc(f) => (10u16, Some(*f), None),
        };

        let frame_bits = match frame_handling {
            Some(FrameChangeHandling::Absolute) => 0u16,
            Some(FrameChangeHandling::Incremental) => 1u16,
            None => 0u16, // Not applicable for Bool
        };

        let positioning_bits = match positioning {
            Some(PercentageNeuronPositioning::Linear) => 0u16,
            Some(PercentageNeuronPositioning::Fractional) => 1u16,
            None => 0u16, // Not applicable for Bool/CartesianPlane/Misc
        };

        // Pack: variant (8 bits) | frame_handling (1 bit)| positioning (1 bit) << 5
        variant | (frame_bits << 8) | (positioning_bits << 9)
    }

    pub const fn as_io_cortical_id(
        &self,
        is_input: bool,
        cortical_unit_identifier: [u8; 3],
        cortical_unit_index: CorticalUnitIndex,
        cortical_sub_unit_index: CorticalSubUnitIndex,
    ) -> CorticalID {
        let data_type_configuration: IOCorticalAreaConfigurationFlagBitmask =
            self.to_data_type_configuration_flag();
        let data_type_configuration_bytes: [u8; 2] = data_type_configuration.to_le_bytes();

        let cortical_id_bytes: [u8; CorticalID::NUMBER_OF_BYTES] = [
            if is_input { b'i' } else { b'o' },
            cortical_unit_identifier[0],
            cortical_unit_identifier[1],
            cortical_unit_identifier[2],
            data_type_configuration_bytes[0],
            data_type_configuration_bytes[1],
            cortical_sub_unit_index.get(),
            cortical_unit_index.get(),
        ];

        CorticalID {
            bytes: cortical_id_bytes,
        }
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
    type Error = FeagiDataError;

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
    pub fn try_from_serde_map(
        map: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<PercentageNeuronPositioning, FeagiDataError> {
        let val = map.get("percentage_neuron_positioning").ok_or(
            FeagiDataError::DeserializationError(
                "Unable to extreact percentage_neuron_positioning!".to_string(),
            ),
        )?;
        let output: PercentageNeuronPositioning =
            serde_json::from_value(val.clone()).map_err(|_err| {
                FeagiDataError::DeserializationError(
                    "Unable to extreact percentage_neuron_positioning!".to_string(),
                )
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FrameChangeHandling {
    #[default]
    Absolute,
    Incremental,
}

impl FrameChangeHandling {
    pub fn try_from_serde_map(
        map: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<FrameChangeHandling, FeagiDataError> {
        let val = map
            .get("frame_change_handling")
            .ok_or(FeagiDataError::DeserializationError(
                "Unable to extreact frame_change_handling!".to_string(),
            ))?;
        let output: FrameChangeHandling = serde_json::from_value(val.clone()).map_err(|_err| {
            FeagiDataError::DeserializationError(
                "Unable to extreact frame_change_handling!".to_string(),
            )
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
//endregion
