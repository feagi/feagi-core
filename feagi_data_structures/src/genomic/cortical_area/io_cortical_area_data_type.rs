use crate::FeagiDataError;
use crate::genomic::cortical_area::CorticalID;
use crate::genomic::cortical_area::descriptors::{CorticalGroupIndex, CorticalUnitIndex};

pub type DataTypeConfigurationFlag = u16;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IOCorticalAreaDataType {
    Percentage(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    CartesianPlane(FrameChangeHandling),
    Misc(FrameChangeHandling)
}

impl IOCorticalAreaDataType {
    pub(crate) fn try_from_data_type_configuration_flag(value: DataTypeConfigurationFlag) -> Result<Self, FeagiDataError> {
        let variant = value & 0x0F; // Bits 0-3
        let frame_handling = (value >> 4) & 0x01; // Bit 4
        let positioning = (value >> 5) & 0x01; // Bit 5

        let frame_handling_enum = match frame_handling {
            0 => FrameChangeHandling::Absolute,
            1 => FrameChangeHandling::Incremental,
            _ => return Err(FeagiDataError::BadParameters("Invalid frame handling value".to_string())),
        };

        let positioning_enum = match positioning {
            0 => PercentageNeuronPositioning::Linear,
            1 => PercentageNeuronPositioning::Fractional,
            _ => return Err(FeagiDataError::BadParameters("Invalid positioning value".to_string())),
        };

        match variant {
            0 => Ok(IOCorticalAreaDataType::Percentage(frame_handling_enum, positioning_enum)),
            1 => Ok(IOCorticalAreaDataType::Percentage2D(frame_handling_enum, positioning_enum)),
            2 => Ok(IOCorticalAreaDataType::Percentage3D(frame_handling_enum, positioning_enum)),
            3 => Ok(IOCorticalAreaDataType::Percentage4D(frame_handling_enum, positioning_enum)),
            4 => Ok(IOCorticalAreaDataType::SignedPercentage(frame_handling_enum, positioning_enum)),
            5 => Ok(IOCorticalAreaDataType::SignedPercentage2D(frame_handling_enum, positioning_enum)),
            6 => Ok(IOCorticalAreaDataType::SignedPercentage3D(frame_handling_enum, positioning_enum)),
            7 => Ok(IOCorticalAreaDataType::SignedPercentage4D(frame_handling_enum, positioning_enum)),
            8 => {
                // CartesianPlane doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::BadParameters("CartesianPlane variant does not support positioning parameter".to_string()));
                }
                Ok(IOCorticalAreaDataType::CartesianPlane(frame_handling_enum))
            }
            9 => {
                // Misc doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::BadParameters("Misc variant does not support positioning parameter".to_string()));
                }
                Ok(IOCorticalAreaDataType::Misc(frame_handling_enum))
            }
            _ => Err(FeagiDataError::BadParameters(format!("Invalid variant type: {}", variant))),
        }
    }

    pub(crate) const fn as_io_cortical_id(&self, is_input: bool, cortical_unit_identifier: [u8; 3], cortical_unit_index: CorticalUnitIndex, cortical_group_index: CorticalGroupIndex) -> CorticalID {

        // TODO there has to be a better method than multiple copies?
        let mut cortical_id_bytes: [u8; CorticalID::NUMBER_OF_BYTES] = {
            if is_input {
                *b"i0000000"
            }
            else {
                *b"00000000"
            }
        };

        cortical_id_bytes[1..4].copy_from_slice(&cortical_unit_identifier);
        let data_type_configuration: DataTypeConfigurationFlag = self.to_data_type_configuration_flag();
        let data_type_configuration_bytes: [u8; 2] = data_type_configuration.to_le_bytes();
        cortical_id_bytes[4..5].copy_from_slice(&data_type_configuration_bytes);
        cortical_id_bytes[6] = *cortical_unit_index;
        cortical_id_bytes[7] = *cortical_group_index;

        CorticalID {
            bytes: cortical_id_bytes,
        }


    }



    pub(crate) const fn to_data_type_configuration_flag(&self) -> DataTypeConfigurationFlag {
        let (variant, frame_handling, positioning) = match self {
            IOCorticalAreaDataType::Percentage(f, p) => (0u16, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage2D(f, p) => (1u16, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage3D(f, p) => (2u16, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage4D(f, p) => (3u16, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage(f, p) => (4u16, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage2D(f, p) => (5u16, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage3D(f, p) => (6u16, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage4D(f, p) => (7u16, *f, Some(*p)),
            IOCorticalAreaDataType::CartesianPlane(f) => (8u16, *f, None),
            IOCorticalAreaDataType::Misc(f) => (9u16, *f, None),
        };

        let frame_bits = match frame_handling {
            FrameChangeHandling::Absolute => 0u16,
            FrameChangeHandling::Incremental => 1u16,
        };

        let positioning_bits = match positioning {
            Some(PercentageNeuronPositioning::Linear) => 0u16,
            Some(PercentageNeuronPositioning::Fractional) => 1u16,
            None => 0u16, // Not applicable for CartesianPlane/Misc
        };

        // Pack: variant (4 bits) | frame_handling (1 bit) << 4 | positioning (1 bit) << 5
        variant | (frame_bits << 4) | (positioning_bits << 5)

        // NOTE: bits 6 and 7 are currently unused, as well as the second byte!
    }
}

impl From<&IOCorticalAreaDataType> for DataTypeConfigurationFlag {
    fn from(data_type: &IOCorticalAreaDataType) -> Self {
        data_type.to_data_type_configuration_flag()
    }
}

impl From<IOCorticalAreaDataType> for DataTypeConfigurationFlag {
    fn from(data_type: IOCorticalAreaDataType) -> Self {
        (&data_type).into()
    }
}

impl TryFrom<DataTypeConfigurationFlag> for IOCorticalAreaDataType {
    type Error = FeagiDataError;

    fn try_from(value: DataTypeConfigurationFlag) -> Result<Self, Self::Error> {
        IOCorticalAreaDataType::try_from_data_type_configuration_flag(value)
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