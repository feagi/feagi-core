

/// All parameters for
pub trait MessageParameters: serde::Serialize + serde::de::DeserializeOwned {
    // All fields must implement `FeagiMessageParameterFieldSerializable`
}

/// To be used on the fields of Parameters for Feagi Message Parameters, to ensure they can be decoded by REST calls
pub trait FeagiMessageParameterFieldSerializable:
core::fmt::Debug
+ core::str::FromStr
+ serde::Serialize
+ serde::de::DeserializeOwned
{
    // TODO does zenoh need anything? probably the const to return the "any" str representation for server side
}


pub trait MessageQueryables: serde::Serialize + serde::de::DeserializeOwned {
    // Mainly only is used directly in rest a URL queryables
}


pub trait MessagePayload: serde::Serialize + serde::de::DeserializeOwned {

}

#[doc(hidden)]
/// In zenoh, we package these together
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(deserialize = "Queryables: serde::de::DeserializeOwned"))]
pub struct MessageQueryablesAndRequest<Queryables: MessageQueryables, Request: MessagePayload> {
    pub queryables: Queryables,
    pub request: Request,
}

// /// Field name -> Doc String. Used mainly by OpenAPI generator
//pub type StaticFieldDocumentation = (&'static str, &'static str);