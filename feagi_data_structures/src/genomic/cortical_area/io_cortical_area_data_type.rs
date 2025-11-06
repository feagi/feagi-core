use crate::FeagiDataError;

pub type DataTypeConfigurationFlag = u8;

#[derive(Debug, PartialEq, Eq)]
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

    /// Converts the enum to a u8 representation.
    ///
    /// Bit layout:
    /// - Bits 0-3: Variant type (0-9)
    /// - Bit 4: FrameChangeHandling (0=Absolute, 1=Incremental)
    /// - Bit 5: PercentageNeuronPositioning (0=Linear, 1=Fractional, only valid for variants 0-7)
    /// - Bits 6-7: Reserved
    ///
    /// This function is `const`, allowing it to be used in compile-time constant expressions.
    pub const fn to_data_type_configuration_flag(&self) -> DataTypeConfigurationFlag {
        let (variant, frame_handling, positioning) = match self {
            IOCorticalAreaDataType::Percentage(f, p) => (0u8, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage2D(f, p) => (1u8, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage3D(f, p) => (2u8, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage4D(f, p) => (3u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage(f, p) => (4u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage2D(f, p) => (5u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage3D(f, p) => (6u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage4D(f, p) => (7u8, *f, Some(*p)),
            IOCorticalAreaDataType::CartesianPlane(f) => (8u8, *f, None),
            IOCorticalAreaDataType::Misc(f) => (9u8, *f, None),
        };

        let frame_bits = match frame_handling {
            FrameChangeHandling::Absolute => 0u8,
            FrameChangeHandling::Incremental => 1u8,
        };

        let positioning_bits = match positioning {
            Some(PercentageNeuronPositioning::Linear) => 0u8,
            Some(PercentageNeuronPositioning::Fractional) => 1u8,
            None => 0u8, // Not applicable for CartesianPlane/Misc
        };

        // Pack: variant (4 bits) | frame_handling (1 bit) << 4 | positioning (1 bit) << 5
        variant | (frame_bits << 4) | (positioning_bits << 5)

        // NOTE: bits 6 and 7 are currently unused!
    }

    /// Attempts to convert an u8 to the enum.
    ///
    /// Returns an error if the variant type is invalid (>9) or if the positioning
    /// bit is set for variants that don't support it (CartesianPlane, Misc).
    pub fn try_from_data_type_configuration_flag(value: DataTypeConfigurationFlag) -> Result<Self, FeagiDataError> {
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