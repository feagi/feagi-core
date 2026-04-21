use crate::FeagiSerializable;
use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::neuron_voxels::coord_potential::CorticalMappedNeuronVoxelCoordVectors;
use feagi_structures::FeagiJSON;
use feagi_structures::FeagiStructuresError;
use std::fmt::{Display, Formatter};

/// Represents different types of serializable data structures in the FEAGI system.
///
/// Each variant corresponds to a specific binary format with a unique byte identifier.
/// The enum values are used as the first byte in serialized data to identify the structure type.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum FeagiByteStructureType {
    /// JSON serialization format (human-readable text).
    JSON = 1u8,

    /// Binary format for cortical-mapped neuron voxel coordinate/potential data.
    ///
    /// The on-wire width of coordinates and potentials depends on the concrete
    /// quantization parameters of the producing/consuming implementation. Two
    /// endpoints must agree on the same `<VoxelPotentialQuant, CoordQuant>`
    /// pairing for a payload to be decodable.
    NeuronCategoricalXYZP = 11u8,
}

impl FeagiByteStructureType {
    /// Determines the structure type from the first byte of a byte array.
    pub fn try_get_type_from_bytes(
        bytes: &[u8],
    ) -> Result<FeagiByteStructureType, FeagiStructuresError> {
        if bytes.is_empty() {
            return Err(FeagiStructuresError::DeserializationError(
                "Cannot ascertain type of empty bytes array!".into(),
            ));
        }
        FeagiByteStructureType::try_from(bytes[0])
    }

    /// Creates a new empty instance of the serializable structure for this type.
    ///
    /// Generic over the four primitive quantization parameters used by the
    /// `MultiCorticalNeuronVoxelCollection*` traits. Callers that have an
    /// `NPUQuantization` in scope should pass `Q::Value`, `Q::Coord`,
    /// `Q::NeuronIndex`, and `Q::CorticalIndex` respectively. Variants that
    /// don't use neuron data (e.g. `JSON`) simply ignore the type parameters.
    pub fn create_new_struct_of_type<V, C, N, A>(&self) -> Box<dyn FeagiSerializable>
    where
        V: QuantizableValueType,
        C: QuantizableUIntType,
        N: QuantizableUIntType,
        A: QuantizableUIntType,
    {
        match self {
            FeagiByteStructureType::NeuronCategoricalXYZP => {
                Box::new(CorticalMappedNeuronVoxelCoordVectors::<V, C, N, A>::new())
            }
            FeagiByteStructureType::JSON => Box::new(FeagiJSON::new_empty()),
        }
    }
}

impl TryFrom<u8> for FeagiByteStructureType {
    type Error = FeagiStructuresError;
    fn try_from(value: u8) -> Result<Self, FeagiStructuresError> {
        match value {
            1 => Ok(FeagiByteStructureType::JSON),
            11 => Ok(FeagiByteStructureType::NeuronCategoricalXYZP),
            _ => Err(FeagiStructuresError::DeserializationError(format!(
                "Unknown FeagiByteStructure type {}",
                value
            ))),
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
