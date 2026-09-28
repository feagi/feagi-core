use core::fmt::Write;
use crate::feagi_error::FeagiErrorTrait;

/// Allows using a type as a RequestResponse Path Parameter, IE "/request/TYPE_IDENTIFIER/get".
pub trait PathParameterEncodable: Sized {
    /// The type of feagi error produced when decoding fails.
    type Error: FeagiErrorTrait; // TODO restrict to request response subtype?? Or have this type do whats needed

    /// Encode `self` as a single URL path segment, writing into `out`.
    fn encode_path_parameter<W: Write>(&self, out: &mut W) -> core::fmt::Result;

    /// Decode `self` from a single URL path segment.
    fn try_from_path_parameter(parameter: &str) -> Result<Self, Self::Error>;
}


/// Allows using a type as a RequestResponse Query value, IE "/request/get?filter=ones".
pub trait PathQueryEncodable: Sized {
    /// The type of feagi error produced when decoding fails.
    type Error: FeagiErrorTrait; // TODO restrict to request response subtype?? Or have this type do whats needed

    /// Encode `self` as a single URL query value, writing into `out`.
    fn encode_path_query<W: Write>(&self, out: &mut W) -> core::fmt::Result;

    /// Decode `self` from a single URL query value.
    fn try_from_path_query(query: &str) -> Result<Self, Self::Error>;
}
