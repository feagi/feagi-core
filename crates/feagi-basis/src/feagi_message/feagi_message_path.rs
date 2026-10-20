use crate::feagi_message::feagi_message_components::FeagiMessageParameterFieldSerializable;

/// Represents the path (and the configurations of it) for a feagi message to take. Zero Sized,
/// ergo has no internal fields
pub trait FeagiMessagePath {
    /// The actual types of each path element
    type TypesTuple; // (&'static str, &'a str, PathParameterElement<P1>, PathParameterElement<P2>)

    /// Just the Configurables as a tuple, needed to make the path instantiation functions inputs
    type ConfigurablesTuple;

    /// The Parameter types as a tuple, needed to make the path instantiation functions inputs
    type ParametersTuple;

    fn as_path_parser_tuple(configurables: Self::ConfigurablesTuple) -> Self::TypesTuple;
}
/// Request Response paths may be used by pico, and thus needs some of its own logic
pub trait FeagiRequestMessagePath: FeagiMessagePath {
    
    type Method: RequestMethodType;
    const METHOD: FeagiMessageEnum = Self::Method::METHOD;
}

/// A path parameter slot carrying only its type `Par`.
pub struct PathParameterElement<Par: FeagiMessageParameterFieldSerializable>(
    core::marker::PhantomData<Par>,
);

impl<Par: FeagiMessageParameterFieldSerializable> PathParameterElement<Par> {
    /// Create a parameter slot marker for type `Par`.
    #[inline]
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

// Hand-written so the impls do not require `Par: Clone`/`Copy`
impl<Par: FeagiMessageParameterFieldSerializable> Clone for PathParameterElement<Par> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Par: FeagiMessageParameterFieldSerializable> Copy for PathParameterElement<Par> {}

impl<Par: FeagiMessageParameterFieldSerializable> core::fmt::Debug for PathParameterElement<Par> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PathParameterElement").finish()
    }
}

impl<Par: FeagiMessageParameterFieldSerializable> Default for PathParameterElement<Par> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

//region Request Method

/// A enum for defining what type of request method is being used.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FeagiMessageEnum {
    Read = 0, // get
    Create = 1, // post
    Edit = 2, // put
    Delete = 3, // delete
    Patch = 4, // patch
}

impl FeagiMessageEnum {
    /// Returns true if a request payload is allowed (Mainly relevant for REST)
    pub const fn is_request_payload_allowed(&self) -> bool {
        !matches!(self, Self::Read)
    }
}

/// A Marker trait for defining what type of request method is being used. Matched to how rest defines it
pub trait RequestMethodType {
    const METHOD: FeagiMessageEnum;
}

/// Requesting getting something. Equal to Rest GET.
/// Note that no payload is allowed in GET rest calls!
pub struct RequestMethodRead;
impl RequestMethodType for RequestMethodRead {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Read;
}

/// Requesting creating something. Equal to Rest POST.
pub struct RequestMethodCreate;
impl RequestMethodType for RequestMethodCreate {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Create;
}

/// Requesting updating something. Equal to Rest PUT.
pub struct RequestMethodEdit;
impl RequestMethodType for RequestMethodEdit {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Edit;
}

/// Requesting getting something. Equal to Rest DELETE.
pub struct RequestMethodDelete;
impl RequestMethodType for RequestMethodDelete {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Delete;
}

/// Requesting getting something. Equal to Rest PATCH.
pub struct RequestMethodPatch;
impl RequestMethodType for RequestMethodPatch {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Patch;
}

//endregion

