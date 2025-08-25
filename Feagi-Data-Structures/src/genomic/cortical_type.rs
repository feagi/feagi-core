use std::fmt;
use crate::basic_components::DimensionRange;
use crate::sensor_definition;
use crate::FeagiDataError;
use crate::genomic::{CorticalID};
use crate::genomic::descriptors::CorticalGroupIndex;
use crate::neurons::NeuronCoderType;

macro_rules! define_io_cortical_types {
    (
        $cortical_io_type_enum_name:ident {
            $(
                $(#[doc = $doc:expr])?
                $cortical_type_key_name:ident => {
                    friendly_name: $display_name:expr,
                    snake_case_identifier: $snake_case_identifier:expr,
                    base_ascii: $base_ascii:expr,
                    channel_dimension_range: $channel_dimension_range:expr,
                    default_coder_type: $default_coder_type:ident,
                }
            ),* $(,)?
        }
    ) => {

        // Type Enum
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $cortical_io_type_enum_name {
            $(
                $(#[doc = $doc])?
                $cortical_type_key_name
            ),*
        }

        impl std::fmt::Display for $cortical_io_type_enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let ch = match self {
                    $(
                        Self::$cortical_type_key_name => $display_name
                    ),*
                };
                write!(f, "'{}'", ch)
            }
        }

        impl $cortical_io_type_enum_name {

            pub fn list_all_sensor_types() -> Vec<$cortical_io_type_enum_name> {
                let mut output: Vec<$cortical_io_type_enum_name> = Vec::new();
                    $(
                        output.push($cortical_io_type_enum_name::$cortical_type_key_name);
                    )*
                output
            }
            
            // Does no cortical ID checking
            pub(crate) fn get_type_from_bytes(id: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<CorticalType, FeagiDataError> {
                let mut id_0: [u8; CorticalID::CORTICAL_ID_LENGTH] = id.clone();
                //id_0.clone_from_slice(id);
                const ZERO_AS_ASCII_BYTE: u8 = 48;
                id_0[4] = ZERO_AS_ASCII_BYTE;
                id_0[5] = ZERO_AS_ASCII_BYTE;

                match &id_0 {
                    $(
                        $base_ascii => Ok((Self::$cortical_type_key_name).into())
                    ),*,
                    _ => return Err(FeagiDataError::InternalError("Failed to map cortical ID to type!".into()))
                }
                

            }

            pub fn to_cortical_id(&self, index: CorticalGroupIndex) -> CorticalID {
                let (high, low) = u8_to_hex_char_u8(*index);
                let mut output: [u8; CorticalID::CORTICAL_ID_LENGTH] =  match self {
                    $(
                        Self::$cortical_type_key_name => *$base_ascii
                    ),*,
                };
                output[4] = high;
                output[5] = low;
                CorticalID {bytes: output} // skip safety checks, we know this is fine
            }
            
            pub fn get_snake_case(&self) -> &str {
                match self {
                    $(
                        Self::$cortical_type_key_name => $snake_case_identifier
                    ),*
                }
            }

            pub fn get_channel_dimension_range(&self) -> DimensionRange {
                match self {
                    $(
                        Self::$cortical_type_key_name => $channel_dimension_range.unwrap()
                    ),*
                }
            }
            
            
        }
    }
}

//region CorticalType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorticalType {
    Custom,
    Memory,
    Core(CoreCorticalType),
    Sensory(SensorCorticalType),
    Motor(MotorCorticalType),
}

impl CorticalType {

    pub fn try_get_type_from_bytes(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<CorticalType, FeagiDataError> {
        let start: u8 = bytes[0];
        match start {
            b'c' => Ok(CorticalType::Custom),
            b'm' => Ok(CorticalType::Memory),
            b'_' => CoreCorticalType::get_type_from_bytes(bytes),
            b'i' => SensorCorticalType::get_type_from_bytes(bytes),
            b'o' => MotorCorticalType::get_type_from_bytes(bytes),
            _ => {
                let as_string = String::from_utf8(bytes.to_vec());
                if as_string.is_err() {
                    return Err(FeagiDataError::DeserializationError("Unable to parse cortical ID bytes as ASCII!".into()));
                }
                Err(FeagiDataError::BadParameters(format!("Invalid cortical ID '{}'!", as_string.unwrap())).into())
            }
        }

    }
    
    pub fn to_cortical_id(&self, io_cortical_index: CorticalGroupIndex) -> Result<CorticalID, FeagiDataError> {
        match self {
            Self::Custom => Err(FeagiDataError::BadParameters("Custom Cortical Areas can have arbitrary Cortical IDs and thus cannot be convert to from type!".into())),
            Self::Memory => Err(FeagiDataError::BadParameters("Memory Cortical Areas can have arbitrary Cortical IDs and thus cannot be convert to from type!".into())),
            Self::Core(c) => {
                Ok(CorticalID::new_core_cortical_area_id(*c)?)
            }
            Self::Sensory(s) => {
                Ok(CorticalID::new_sensor_cortical_area_id(*s, io_cortical_index)?)
            }
            Self::Motor(m) => {
                Ok(CorticalID::new_motor_cortical_area_id(*m, io_cortical_index)?)
            }
        }

    }
    
    pub fn try_get_channel_size_boundaries(&self) -> Result<DimensionRange, FeagiDataError> {
        match self {
            Self::Custom => Err(FeagiDataError::BadParameters("Custom Cortical Areas do not have channels!".into())),
            Self::Memory => Err(FeagiDataError::BadParameters("Memory Cortical Areas do not have channels!".into())),
            Self::Core(_) => Err(FeagiDataError::BadParameters("Core Cortical Areas do not have channels!".into())),
            Self::Sensory(s) => Ok(s.get_channel_dimension_range()),
            Self::Motor(m) => Ok(m.get_channel_dimension_range()),
        }
    }
    
    /*
    pub(crate) fn try_get_coder_type(&self) -> Result<NeuronCoderVariantType, FeagiDataError> {
        match self {
            Self::Custom => Err(FeagiDataError::BadParameter("Custom Cortical Areas do not have coders!".into())),
            Self::Memory => Err(FeagiDataError::BadParameter("Memory Cortical Areas do not have coders!".into())),
            Self::Core(_) => Err(FeagiDataError::BadParameter("Core Cortical Areas do not have coders!".into())),
            Self::Sensory(s) => s.get_coder_type(),
            Self::Motor(m) => m.get_coder_type(),
        }
    }
     */
    
    //region Is Type

    pub fn is_type_core(&self) -> bool {
        match self {
            Self::Core(_) => true,
            _ => false
        }
    }


    pub fn is_type_sensor(&self) -> bool {
        match self {
            Self::Sensory(_) => true,
            _ => false
        }
    }


    pub fn is_type_motor(&self) -> bool {
        match self {
            Self::Motor(_) => true,
            _ => false
        }
    }


    pub fn is_type_custom(&self) -> bool {
        match self {
            Self::Custom => true,
            _ => false
        }
    }


    pub fn is_type_memory(&self) -> bool {
        match self {
            Self::Memory => true,
            _ => false
        }
    }
    
    //endregion
    
    //region Verify Type
    pub fn verify_is_core(&self) -> Result<(), FeagiDataError> {
        if !self.is_type_core() {
            return Err(FeagiDataError::BadParameters("Expected cortical type to be type Core!".into()))
        }
        Ok(())
    }


    pub fn verify_is_sensor(&self) -> Result<(), FeagiDataError> {
        if !self.is_type_sensor() {
            return Err(FeagiDataError::BadParameters("Expected cortical type to be type Sensor!".into()))
        }
        Ok(())
    }


    pub fn verify_is_motor(&self) -> Result<(), FeagiDataError> {
        if !self.is_type_motor() {
            return Err(FeagiDataError::BadParameters("Expected cortical type to be type Motor!".into()))
        }
        Ok(())
    }


    pub fn verify_is_custom(&self) -> Result<(), FeagiDataError> {
        if !self.is_type_custom() {
            return Err(FeagiDataError::BadParameters("Expected cortical type to be type Custom!".into()))
        }
        Ok(())
    }


    pub fn verify_is_memory(&self) -> Result<(), FeagiDataError> {
        if !self.is_type_memory() {
            return Err(FeagiDataError::BadParameters("Expected cortical type to be type Memory!".into()))
        }
        Ok(())
    }
    //endregion
}

impl fmt::Display for CorticalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom => write!(f, "'Custom'"),
            Self::Memory => write!(f, "'Memory'"),
            Self::Core(c) => write!(f, "'Core({})'", c),
            Self::Sensory(s) => write!(f, "'Sensory({})'", s),
            Self::Motor(m) => write!(f, "'Motor({})'", m),
        }
    }
}
//endregion

//region Sensor Cortical Area types

// Use sensor_definition directly with callback pattern
sensor_definition!(define_io_cortical_types);

impl From<SensorCorticalType> for CorticalType {
    fn from(input: SensorCorticalType) -> Self {
        CorticalType::Sensory(input)
    }
}

impl TryFrom<CorticalType> for SensorCorticalType {
    type Error = &'static str;
    fn try_from(input: CorticalType) -> Result<Self, Self::Error> {
        match input { 
            CorticalType::Sensory(c) => Ok(c),
            _ => Err("CorticalType is not a SensorCorticalType!")
        }
    }
}

//endregion

//region Motor Cortical Area types

define_io_cortical_types!{
    MotorCorticalType {
        RotaryMotor => {
            friendly_name: "Rotary Motor",
            snake_case_identifier: "rotary_motor",
            base_ascii: b"omot00",
            channel_dimension_range: DimensionRange::new(1..2, 1..2, 1..u32::MAX),
            default_coder_type: F32NormalizedM1To1_SplitSignDivided,
        },
    }    
}

impl From<MotorCorticalType> for CorticalType {
    fn from(input: MotorCorticalType) -> Self {
        CorticalType::Motor(input)
    }
}

impl TryFrom<CorticalType> for MotorCorticalType {
    type Error = &'static str;
    fn try_from(input: CorticalType) -> Result<Self, Self::Error> {
        match input { 
            CorticalType::Motor(c) => Ok(c),
            _ => Err("CorticalType is not a MotorCorticalType!")
        }
    }
}

//endregion

//region CoreCorticalType
// This won't be expanded, this doesn't need a template

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreCorticalType {
    Death,
    Power
}

impl fmt::Display for CoreCorticalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = match self {
            CoreCorticalType::Death => "Death",
            CoreCorticalType::Power => "Power"
        };
        write!(f, "CoreCorticalType({})", ch)
    }
}

impl From<CoreCorticalType> for CorticalType {
    fn from(core: CoreCorticalType) -> Self {
        CorticalType::Core(core)
    }
}

impl From<&CoreCorticalType> for CorticalType {
    fn from(core: &CoreCorticalType) -> Self {
        CorticalType::Core(*core)
    }
}

impl CoreCorticalType {
    pub fn to_cortical_id(&self) -> CorticalID {
        match self {
            Self::Death => CorticalID{bytes: *b"_death"},
            Self::Power => CorticalID{bytes: *b"_power"},
        }
    }

    pub(crate) fn get_type_from_bytes(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<CorticalType, FeagiDataError> {
        match bytes {
            b"_death" => Ok(CoreCorticalType::Death.into()),
            b"_power" => Ok(CoreCorticalType::Power.into()),
            _ => Err(FeagiDataError::InternalError("Failed to deserialize bytes!".into())),
        }
    }

}
//endregion

//region Internal
fn hex_chars_to_u8(high: char, low: char) -> Result<u8, FeagiDataError> {
    fn hex_value(c: char) -> Result<u8, FeagiDataError> {
        match c {
            '0'..='9' => Ok(c as u8 - b'0'),
            'a'..='f' => Ok(c as u8 - b'a' + 10),
            'A'..='F' => Ok(c as u8 - b'A' + 10),
            _ => Err(FeagiDataError::DeserializationError(format!("Index of '{}' is not a valid hexadecimal!", c))),
        }
    }
    let hi = hex_value(high)?;
    let lo = hex_value(low)?;

    Ok((hi << 4) | lo)
}

fn u8_to_hex_char_u8(index: u8) -> (u8, u8) {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

    let high = HEX_CHARS[(index >> 4) as usize];
    let low = HEX_CHARS[(index & 0x0F) as usize];

    (high, low)
}

//endregion