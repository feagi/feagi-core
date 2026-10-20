macro_rules! impl_trait_field_serializable {
    ($marker_trait:ident, $($field_type:ty),*) => {
        $( impl $marker_trait for $field_type {} )*
    };
}

//region Parameters

/// All Parameters for a message. These allow dynamic intra-path specifications 
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


impl_trait_field_serializable!(FeagiMessageParameterFieldSerializable, bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

#[cfg(feature = "alloc")]
impl FeagiMessageParameterFieldSerializable for alloc::string::String {}

impl MessageParameters for () {}

//endregion

//region Queryables

/// These are serializable variables that in most cases are sent alongside the payload,
/// but in the special case of get requests in picoserve they are put in the URL
pub trait MessageQueryables: serde::Serialize + serde::de::DeserializeOwned {

}

impl MessageQueryables for () {}

//endregion

//region Request Payload

pub trait MessagePayload: serde::Serialize + serde::de::DeserializeOwned {

}

// Messages without any parameters, queryables, or payload use the unit type


impl MessagePayload for () {}

//endregion

#[doc(hidden)]
/// In zenoh, we package these together
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(deserialize = "Queryables: serde::de::DeserializeOwned"))]
pub struct MessageQueryablesAndRequest<Queryables: MessageQueryables, Request: MessagePayload> {
    pub queryables: Queryables,
    pub request: Request,
}
