
/// The first step for a message path, allows parsing in the configrables as a first step
pub trait FeagiMessagePathPrebuilder {
    /// A tuple of some length of `FeagiMessagePathConfigurablesElement` instances, one for and in
    /// order of when they occur in the path
    type Configurables;

    fn to_message_path_builder<Configurables: >
}



pub struct FeagiMessagePathStaticElement<const LENGTH: usize>(heapless::String<LENGTH>);

impl<const LENGTH: usize> FeagiMessagePathStaticElement<LENGTH> {
    pub const fn new_from_static_str() -> Self {
        const L: usize =
    }
}


/// A path segment that is not known at compile time, but is set once at runtime and does not
/// change per request (for example a server name read from a config file)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FeagiMessagePathConfigurablesElement<const LENGTH: usize>(heapless::String<LENGTH>);

impl<const LENGTH: usize> FeagiMessagePathConfigurablesElement<LENGTH> {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<const LENGTH: usize> TryFrom<&str> for FeagiMessagePathConfigurablesElement<LENGTH> {
    type Error = FeagiPathConfigurableError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        if text.is_empty() {
            return Err(FeagiPathConfigurableError::Empty);
        }
        if text.len() != LENGTH {
            return Err(FeagiPathConfigurableError::TooLong {
                actual: text.len(),
                target: LENGTH,
            });
        }
        let text = heapless::String::try_from(text)
            .unwrap_or_else(|_| panic!("text length was checked against MAX_LENGTH"));
        Ok(Self(text))
    }
}

impl<const LENGTH: usize> core::fmt::Display for FeagiMessagePathConfigurablesElement<LENGTH> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Text could not be used as a `FeagiPathConfigurable`
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FeagiPathConfigurableError {
    #[error("Path configurable cannot be empty!")]
    Empty,
    #[error("Path configurable is {actual} characters, when it should be {target}!")]
    TooLong { actual: usize, target: usize },
}