use serde::{Deserialize, Serialize};
use feagi_basis::feagi_genome::genome_structures::cortical_areas::cortical_id::CorticalID;

/// Represents data being passed to or from a connected system
pub trait CorrespondenceData<'de>: Clone + PartialEq + Serialize + Deserialize<'de> {

    #[cfg(feature = "support_json")]
    fn from_json(json: serde_json::Value) -> Result<Self, ()>;

    #[cfg(feature = "support_json")]
    fn to_json(&self) -> serde_json::Value;
}

// TODO
/*
/// Marker trait for any data that is making a request
pub trait RequestData<'de>: CorrespondenceData<'de> {}

/// Marker trait for any data that is making a response
pub trait ResponseData<'de>: CorrespondenceData<'de> {}

 */

// TODO custom derive macro (single field vs struct)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Test {
    pub data: Vec<CorticalID>
}

impl<'de> CorrespondenceData<'de> for Test {

    #[cfg(feature = "support_json")]
    fn from_json(json: serde_json::Value) -> Result<Self, ()> {
        let result = serde_json::from_value(json);
        match result {
            Ok(data) => Ok(data),
            Err(_) => Err(())
        }
    }

    #[cfg(feature = "support_json")]
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}


