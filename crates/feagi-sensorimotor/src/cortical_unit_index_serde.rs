use feagi_genomic_context::cortical_unit::CorticalUnitIndex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(value: &CorticalUnitIndex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.deref().serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<CorticalUnitIndex, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    Ok(CorticalUnitIndex::new(value))
}
