use std::fmt::{Display, Formatter};

/// Represents different types of serializable data structures in the FEAGI system.
///
/// Each variant corresponds to a specific binary format with a unique byte identifier.
/// The enum values are used as the first byte in serialized data to identify the structure type.
///
/// # Example
/// ```
/// use feagi_serialization::FeagiByteStructureType;
///
/// let json_type = FeagiByteStructureType::JSON;
/// assert_eq!(json_type as u8, 1);
///
/// let neuron_type = FeagiByteStructureType::NeuronCategoricalXYZP;
/// assert_eq!(neuron_type as u8, 11);
/// ```
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum FeagiByteStructureType {
    /// JSON serialization format (human-readable text)
    JSON = 1u8,

    /// Binary format for neuron categorical XYZP data.
    ///
    /// Binary format specifically designed for neuron data
    /// with X, Y, Z coordinates and potential (P) values.
    NeuronCategoricalXYZP = 11u8,
}

impl FeagiByteStructureType {
    /*
    /// Determines the structure type from the first byte of a byte array.
    ///
    /// # Example
    /// ```
    /// use feagi_serialization::FeagiByteStructureType;
    ///
    /// let bytes = [1u8, 2, 3, 4];
    /// let structure_type = FeagiByteStructureType::try_get_type_from_bytes(&bytes).unwrap();
    /// assert_eq!(structure_type, FeagiByteStructureType::JSON);
    ///
    /// let empty_bytes = [];
    /// assert!(FeagiByteStructureType::try_get_type_from_bytes(&empty_bytes).is_err());
    /// ```
    pub fn try_get_type_from_bytes(bytes: &[u8]) -> Result<FeagiByteStructureType, ()> {
        if bytes.is_empty() {
            return Err(());
            /*
            return Err(FeagiDataError::DeserializationError(
                "Cannot ascertain type of empty bytes array!".into(),
            ));

             */
        }
        FeagiByteStructureType::try_from(bytes[0])
    }

     */
}

impl TryFrom<u8> for FeagiByteStructureType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(FeagiByteStructureType::JSON),
            11 => Ok(FeagiByteStructureType::NeuronCategoricalXYZP),
            _ => Err(()),
            //_ => Err(FeagiDataError::DeserializationError(format!("Unknown FeagiByteStructure type {}", value))),
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
