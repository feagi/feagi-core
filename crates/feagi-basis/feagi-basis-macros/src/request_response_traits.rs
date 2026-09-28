use crate::feagi_error::FeagiErrorTrait;

/// Allows using a type as a RequestResponse Path Parameter, IE "/request/TYPE_IDENTIFIER/get"
pub trait PathParameterEncodable: Sized {
    /// The type of feagi error to encode for
    type Error: FeagiErrorTrait; // TODO restrict to request response subtype?? Or have this type do whats needed
    
    fn to_path_parameter(&self) -> &str;
    
    fn try_from_path_parameter(parameter: &str) -> Result<Self, Self::Error>;
}


/// Allows using a type as a RequestResponse Query, IE "/request/get?filter=ones"
pub trait PathQueryEncodable: Sized {
    /// The type of feagi error to encode for
    type Error: FeagiErrorTrait; // TODO restrict to request response subtype?? Or have this type do whats needed

    fn to_path_query(&self) -> &str;

    fn try_from_path_query(query: &str) -> Result<Self, Self::Error>;
}