use std::fmt::{Display, Formatter};
use feagi_data_structures::FeagiDataError;


#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum FeagiByteStructureType {
    /// JSON serialization format (human-readable text)
    JSON = 1u8,

    /// Binary format for neuron categorical XYZP data.
    /// 
    /// Binary format specifically designed for neuron dataW
    /// with X, Y, Z coordinates and potential (P) values.
    NeuronCategoricalXYZP = 11u8
}

impl FeagiByteStructureType {
    pub fn try_get_type_from_bytes(bytes: &[u8]) -> Result<FeagiByteStructureType, FeagiDataError> {
        if bytes.len() < 1 {
            return Err(FeagiDataError::DeserializationError("Cannot ascertain type of empty bytes array!".into()).into())
        }
        FeagiByteStructureType::try_from(bytes[0])
    }

}

impl From<FeagiByteStructureType> for u8 {
    fn from(value: FeagiByteStructureType) -> u8 {
        value as u8
    }
}

impl TryFrom<u8> for FeagiByteStructureType {
    type Error = FeagiDataError;
    fn try_from(value: u8) -> Result<Self, FeagiDataError> {
        match value {
            1 => Ok(FeagiByteStructureType::JSON),
            11 => Ok(FeagiByteStructureType::NeuronCategoricalXYZP),
            _ => Err(FeagiDataError::DeserializationError(format!("Unknown FeagiByteStructure type {}", value)).into())
        }
    }
}

impl Display for FeagiByteStructureType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FeagiByteStructureType::JSON => "JSON",
            FeagiByteStructureType::NeuronCategoricalXYZP => "NeuronCategoricalXYZP",
        };
        write!(f, "{name}")
    }
}