

// TODO Feature Gating

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

}

/// The allowed types for the tuple of FeagiMessagePath
pub mod path_element_type {
    use crate::feagi_message::feagi_message_components::FeagiMessageParameterFieldSerializable;

    /// Constant parts of a path that are constant (at least by compile time)
    pub type PathStaticElement = &'static str;

    /// A part of a path that is configurable only at the path instantiation
    pub type PathConfigurableElement<'a> = &'a str;


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





