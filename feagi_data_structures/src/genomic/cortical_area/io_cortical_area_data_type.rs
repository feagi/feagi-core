use crate::FeagiDataError;
use crate::genomic::cortical_area::CorticalID;
use crate::genomic::cortical_area::descriptors::{CorticalGroupIndex, CorticalUnitIndex};


pub type DataTypeConfigurationFlag = u16; // 16 Total bits

// Bits 0-7 -> Enum
// Bit 8 -> FrameChangeHandling
// Bit 9 -> PercentageNeuronPositioning
// Bit 10-15 -> RESERVED

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IOCorticalAreaDataFlag {
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

impl IOCorticalAreaDataFlag {
    pub const fn try_from_data_type_configuration_flag(value: DataTypeConfigurationFlag) -> Result<Self, FeagiDataError> {
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
            0 => Ok(IOCorticalAreaDataFlag::Boolean),
            1 => Ok(IOCorticalAreaDataFlag::Percentage(frame_handling_enum, positioning_enum)),
            2 => Ok(IOCorticalAreaDataFlag::Percentage2D(frame_handling_enum, positioning_enum)),
            3 => Ok(IOCorticalAreaDataFlag::Percentage3D(frame_handling_enum, positioning_enum)),
            4 => Ok(IOCorticalAreaDataFlag::Percentage4D(frame_handling_enum, positioning_enum)),
            5 => Ok(IOCorticalAreaDataFlag::SignedPercentage(frame_handling_enum, positioning_enum)),
            6 => Ok(IOCorticalAreaDataFlag::SignedPercentage2D(frame_handling_enum, positioning_enum)),
            7 => Ok(IOCorticalAreaDataFlag::SignedPercentage3D(frame_handling_enum, positioning_enum)),
            8 => Ok(IOCorticalAreaDataFlag::SignedPercentage4D(frame_handling_enum, positioning_enum)),
            9 => {
                // CartesianPlane doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::ConstError("CartesianPlane variant does not support positioning parameter"));
                }
                Ok(IOCorticalAreaDataFlag::CartesianPlane(frame_handling_enum))
            }
            10 => {
                // Misc doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::ConstError("Misc variant does not support positioning parameter"));
                }
                Ok(IOCorticalAreaDataFlag::Misc(frame_handling_enum))
            }
            _ => Err(FeagiDataError::ConstError("Invalid variant type!")),
        }
    }

    pub const fn to_data_type_configuration_flag(&self) -> DataTypeConfigurationFlag {
        let (variant, frame_handling, positioning) = match self {
            IOCorticalAreaDataFlag::Boolean => (0u16, None, None),
            IOCorticalAreaDataFlag::Percentage(f, p) => (0u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::Percentage2D(f, p) => (1u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::Percentage3D(f, p) => (2u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::Percentage4D(f, p) => (3u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::SignedPercentage(f, p) => (4u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::SignedPercentage2D(f, p) => (5u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::SignedPercentage3D(f, p) => (6u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::SignedPercentage4D(f, p) => (7u16, Some(*f), Some(*p)),
            IOCorticalAreaDataFlag::CartesianPlane(f) => (8u16, Some(*f), None),
            IOCorticalAreaDataFlag::Misc(f) => (9u16, Some(*f), None),
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

    pub const fn as_io_cortical_id(&self, is_input: bool, cortical_unit_identifier: [u8; 3], cortical_unit_index: CorticalUnitIndex, cortical_group_index: CorticalGroupIndex) -> CorticalID {
        let data_type_configuration: DataTypeConfigurationFlag = self.to_data_type_configuration_flag();
        let data_type_configuration_bytes: [u8; 2] = data_type_configuration.to_le_bytes();

        let cortical_id_bytes: [u8; CorticalID::NUMBER_OF_BYTES] = [
            if is_input { b'i' } else { b'o' },
            cortical_unit_identifier[0],
            cortical_unit_identifier[1],
            cortical_unit_identifier[2],
            data_type_configuration_bytes[0],
            data_type_configuration_bytes[1],
            cortical_unit_index.get(),
            cortical_group_index.get(),
        ];

        CorticalID {
            bytes: cortical_id_bytes,
        }
    }

}

impl From<&IOCorticalAreaDataFlag> for DataTypeConfigurationFlag {
    fn from(data_type: &IOCorticalAreaDataFlag) -> Self {
        data_type.to_data_type_configuration_flag()
    }
}

impl From<IOCorticalAreaDataFlag> for DataTypeConfigurationFlag {
    fn from(data_type: IOCorticalAreaDataFlag) -> Self {
        (&data_type).into()
    }
}

impl TryFrom<DataTypeConfigurationFlag> for IOCorticalAreaDataFlag {
    type Error = FeagiDataError;

    fn try_from(value: DataTypeConfigurationFlag) -> Result<Self, Self::Error> {
        IOCorticalAreaDataFlag::try_from_data_type_configuration_flag(value)
    }
}

//region SubEnums
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum PercentageNeuronPositioning {
    Linear,
    #[default]
    Fractional,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum FrameChangeHandling {
    #[default]
    Absolute,
    Incremental,
}
//endregion