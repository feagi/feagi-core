use crate::feagi_message::feagi_message_components::FeagiMessageParameterFieldSerializable;

/// Represents the path (and the configurations of it) for a feagi message to take. Zero Sized,
/// ergo has no internal fields
pub trait FeagiMessagePath {
    /// The runtime set path segments of the category this path belongs to
    type Configurables;

    /// The actual types of each path element, starting with the category's base path. Static
    /// segments are `&'static str`, configurable segments borrow from `Configurables`
    type TypesTuple<'a> // (&'static str, &'a str, PathParameterElement<P1>, PathParameterElement<P2>)
    where
        Self::Configurables: 'a;

    /// The Parameter types as a tuple, needed to make the path instantiation functions inputs
    type ParametersTuple;

    fn as_path_parser_tuple(configurables: &Self::Configurables) -> Self::TypesTuple<'_>;
}





//region Path Parameter Element

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

//endregion

//region Path Element Configurable

/// A path segment that is not known at compile time, but is set once at runtime and does not
/// change per request (for example a server name read from a config file). Holds at most
/// `MAX_LENGTH` characters of the URL-safe base64 alphabet (`A-Z`, `a-z`, `0-9`, `-`, `_`, `=`),
/// so it is always a valid single segment of a REST path or zenoh key
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FeagiPathElementConfigurable<const MAX_LENGTH: usize>(heapless::String<MAX_LENGTH>);

impl<const MAX_LENGTH: usize> FeagiPathElementConfigurable<MAX_LENGTH> {
    /// Errors if `text` is empty, longer than `MAX_LENGTH`, or has a character outside of the
    /// URL-safe base64 alphabet
    pub fn try_from_str(text: &str) -> Result<Self, FeagiPathConfigurableError> {
        if text.is_empty() {
            return Err(FeagiPathConfigurableError::Empty);
        }
        if text.len() > MAX_LENGTH {
            return Err(FeagiPathConfigurableError::TooLong {
                actual: text.len(),
                max: MAX_LENGTH,
            });
        }
        if let Some(invalid) = text.chars().find(|ch| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '='))) {
            return Err(FeagiPathConfigurableError::InvalidCharacter(invalid));
        }
        let text = heapless::String::try_from(text).unwrap_or_else(|_| panic!("text length was checked against MAX_LENGTH"));
        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<const MAX_LENGTH: usize> TryFrom<&str> for FeagiPathElementConfigurable<MAX_LENGTH> {
    type Error = FeagiPathConfigurableError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(text)
    }
}

impl<const MAX_LENGTH: usize> core::fmt::Display for FeagiPathElementConfigurable<MAX_LENGTH> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Text could not be used as a `FeagiPathConfigurable`
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FeagiPathConfigurableError {
    #[error("Path configurable cannot be empty!")]
    Empty,
    #[error("Path configurable is {actual} characters, more than its maximum of {max}!")]
    TooLong { actual: usize, max: usize },
    #[error("Path configurable has '{0}', which is not in the URL-safe base64 alphabet!")]
    InvalidCharacter(char),
}

//endregion

