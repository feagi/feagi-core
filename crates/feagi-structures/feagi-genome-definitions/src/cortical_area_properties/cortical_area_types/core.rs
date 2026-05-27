use crate::cortical_area_properties::cortical_id::CorticalIDPacked;
use crate::feagi_genome_definition_error::{CorticalIDLookupErrKey, FeagiGenomeDefinitionsError};

/// Core cortical area types for fundamental brain functions.
///
/// Represents essential processing regions that manage the agent's power,
/// termination states, and fatigue monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreCorticalType {
    /// Termination/death signal processing
    Death,
    /// Power management processing
    Power,
    /// Brain fatigue indicator - activates when neuron/synapse arrays exceed 85% capacity
    Fatigue,
    /// Pain signal processing
    Pain,
    /// Pleasure signal processing
    Pleasure,
    /// Fear signal processing
    Fear,
    /// Hope signal processing
    Hope,
}

impl CoreCorticalType {
    
    // TODO store the possible types as consts for easy access in other crates
    
    pub const fn to_cortical_identifier_packed(&self) -> CorticalIDPacked {
        match &self {
            CoreCorticalType::Death => {CorticalIDPacked::new_const_unchecked(*b"___death")}
            CoreCorticalType::Power => {CorticalIDPacked::new_const_unchecked(*b"___power")}
            CoreCorticalType::Fatigue => {CorticalIDPacked::new_const_unchecked(*b"___fatig")}
            CoreCorticalType::Pain => {CorticalIDPacked::new_const_unchecked(*b"___pain_")}
            CoreCorticalType::Pleasure => {CorticalIDPacked::new_const_unchecked(*b"___pleas")}
            CoreCorticalType::Fear => {CorticalIDPacked::new_const_unchecked(*b"___fear_")}
            CoreCorticalType::Hope => {CorticalIDPacked::new_const_unchecked(*b"___hope_")}
        }
    }
    
    
    pub fn try_from_cortical_id_bytes_type_unchecked(
        cortical_id_packed_bytes: &[u8; CorticalIDPacked::BYTE_COUNT],
    ) -> Result<CoreCorticalType, FeagiGenomeDefinitionsError> {
        match cortical_id_packed_bytes {
            b"___death" => Ok(CoreCorticalType::Death),
            b"___power" => Ok(CoreCorticalType::Power),
            b"___fatig" => Ok(CoreCorticalType::Fatigue),
            b"___pain_" => Ok(CoreCorticalType::Pain),
            b"___pleas" => Ok(CoreCorticalType::Pleasure),
            b"___fear_" => Ok(CoreCorticalType::Fear),
            b"___hope_" => Ok(CoreCorticalType::Hope),
            _ => Err(CorticalIDLookupErrKey::new(
                "Unable to cast given cortical ID bytes to a known core cortical type!",
                cortical_id_packed_bytes.clone()
            ).into())
        }
    }

    pub fn to_cortical_id(&self) -> CorticalIDPacked {
        match self {
            Self::Death => CorticalIDPacked::new_const_unchecked(*b"___death"),
            Self::Power => CorticalIDPacked::new_const_unchecked(*b"___power"),
            Self::Fatigue => CorticalIDPacked::new_const_unchecked(*b"___fatig"),
            Self::Pain => CorticalIDPacked::new_const_unchecked(*b"___pain_"),
            Self::Pleasure => CorticalIDPacked::new_const_unchecked(*b"___pleas"),
            Self::Fear => CorticalIDPacked::new_const_unchecked(*b"___fear_"),
            Self::Hope => CorticalIDPacked::new_const_unchecked(*b"___hope_")
        }
    }
}