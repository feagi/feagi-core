
/// The path a message takes. Zero sized, the path is described entirely by its types and
/// associated items, for parsers in zenoh / picoserve / open-api to generate what they need
pub trait FeagiMessagePath: 'static {

    /// A struct of `FeagiMessagePathStaticElement` instances, shared by the category
    type Configurables: configurables::FeagiMessagePathConfigurables;

    /// A struct of `FeagiMessagePathParameterElement` instances used in this path.
    type Parameters: parameters::FeagiMessagePathParameters;

    /// A flat tuple of the whole path as known at compile time: &'static str for static path
    /// components, `ConfigurableSlot` where a configurable goes, and `MessagePathSlotElement`
    /// where a parameter goes
    type FullMessagePathBaseTuple;

    /// The whole path as known at compile time. Usable in const code (open-api)
    const FULL_MESSAGE_PATH: Self::FullMessagePathBaseTuple;

    /// The flat tuple representation of the path once configurables are applied: &'static str
    /// for static path components and configurable values, and `MessagePathSlotElement` for the
    /// parameters that must be parsed dynamically
    type SlottedParameterMessagePath;

    /// Applies the configurables to `FULL_MESSAGE_PATH`, giving `SlottedParameterMessagePath`.
    /// Configurables are set once and live as long as the program, hence `'static`
    fn to_slotted_parameter_message_path(configurables: &'static Self::Configurables) -> Self::SlottedParameterMessagePath;

    /// Writes the concrete path for a request with the given `parameters`, segments separated by
    /// `/`. Used by the requesting side (zenoh keys, REST client URLs)
    fn write_message_path<W: core::fmt::Write>(
        configurables: &Self::Configurables,
        parameters: &Self::Parameters,
        out: &mut W,
    ) -> core::fmt::Result;
}


/// Defines something that can be serialized to become part of the message path statically.
/// Think reading server name from a config file
pub mod configurables {
    /// Struct composed of various `FeagiMessagePathStaticElement` fields. Implemented by
    /// macro
    pub trait FeagiMessagePathConfigurables {
        // Composed of pub fields of `FeagiMessagePathStaticElement`
    }

    // Categories without any configurables use the unit type
    impl FeagiMessagePathConfigurables for () {}


    /// Defines something that can be serialized to become part of the message path statically.
    /// Used for allowing things to be used as `FeagiMessagePathConfigurables` fields
    pub trait FeagiMessagePathStaticElement {
        fn to_str_slice(&self) -> &str;
    }


    impl FeagiMessagePathStaticElement for &'static str {
        fn to_str_slice(&self) -> &str {
            self
        }
    }

    impl<const N: usize> FeagiMessagePathStaticElement for heapless::String<N> {
        fn to_str_slice(&self) -> &str {
            self.as_str()
        }
    }

    #[cfg(feature = "alloc")]
    impl FeagiMessagePathStaticElement for String {
        fn to_str_slice(&self) -> &str {
            self.as_str()
        }
    }

    /// Where a configurable goes in `FeagiMessagePath::FULL_MESSAGE_PATH`. Its value is only known
    /// at runtime, so only its name is held
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConfigurableSlot {
        /// Name of the configurable, as its field in the category's configurables struct
        pub name: &'static str,
    }

    impl ConfigurableSlot {
        pub const fn new(name: &'static str) -> Self {
            Self { name }
        }
    }
}

/// Dynamic path elements that may be dynamic per call. Includes information that is  part of
/// the Feagi message overall
pub mod parameters {

    /// Struct composed of various `FeagiMessagePathParameterElement` fields. Implemented by
    /// macro. Not used directly by FeagiMessagePath, but still useful as reference
    pub trait FeagiMessagePathParameters {
        // Composed of pub fields of `FeagiMessagePathParameterElement`
    }

    // Messages without any path parameters use the unit type
    impl FeagiMessagePathParameters for () {}

    /// Something that can be serialized to become part of the message path dynamically as
    /// a parameter.
    pub trait FeagiMessagePathParameterElement: core::str::FromStr + core::fmt::Debug
    + serde::Serialize + serde::de::DeserializeOwned
    {
        /// Writes this parameter as its path segment. What is written must parse back into the
        /// same value through `FromStr`
        fn write_path_segment<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result;
    }

    /// Primitives are written as they display, which `FromStr` parses back
    macro_rules! impl_parameter_element_by_display {
        ($($element_type:ty),*) => {
            $(
                impl FeagiMessagePathParameterElement for $element_type {
                    fn write_path_segment<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
                        write!(out, "{self}")
                    }
                }
            )*
        };
    }

    impl_parameter_element_by_display!(bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

    #[cfg(feature = "alloc")]
    impl FeagiMessagePathParameterElement for String {
        fn write_path_segment<W: core::fmt::Write>(&self, out: &mut W) -> core::fmt::Result {
            out.write_str(self.as_str())
        }
    }

    //region Path Parameter Slot

    /// A path parameter slot carrying its name and type `Par`, but no value. Used for building
    /// stuff via the type system
    pub struct MessagePathSlotElement<Par: FeagiMessagePathParameterElement> {
        /// Name of the parameter, as its field in the message's parameters struct
        pub name: &'static str,
        _parameter: core::marker::PhantomData<Par>,
    }

    impl<Par: FeagiMessagePathParameterElement> MessagePathSlotElement<Par> {
        /// Create a parameter slot marker named `name` for type `Par`.
        #[inline]
        pub const fn new(name: &'static str) -> Self {
            Self {
                name,
                _parameter: core::marker::PhantomData,
            }
        }
    }

    // Hand-written so the impls do not require `Par: Clone`/`Copy`
    impl<Par: FeagiMessagePathParameterElement> Clone for MessagePathSlotElement<Par> {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<Par: FeagiMessagePathParameterElement> Copy for MessagePathSlotElement<Par> {}

    impl<Par: FeagiMessagePathParameterElement> core::fmt::Debug for MessagePathSlotElement<Par> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("MessagePathSlotElement").field("name", &self.name).finish()
        }
    }

    //endregion
    
}
