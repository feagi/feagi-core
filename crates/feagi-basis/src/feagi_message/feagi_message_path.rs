
/// The first step for a message path, allows parsing in the configrables as a first step
pub trait FeagiMessagePath: 'static {

    /// A flat tuple with &'static str for static path components, `FeagiMessagePathConfigurables`
    /// to be overwritten by configurables, and `FeagiMessagePathParameters` wrapped by
    /// `MessagePathSlotElement`
    type FullMessagePathBaseTuple;

    /// A struct of `FeagiMessagePathConfigurablesElement` instances
    type Configurables: configurables::FeagiMessagePathConfigurables;

    /// A struct of `FeagiMessagePathParameterElement` instances used in this path.
    type Parameters: parameters::FeagiMessagePathParameters;

    /// The flat tuple representation of the path containing types of &str where we have defined
    /// elements of the path (either statically or from parsed configurables) and
    /// `MessagePathSlotElement` defines the various Parameters that must be parsed dynamically
    type SlottedParameterMessagePath;

    /// Applies the configurables to `FullMessagePathBaseTuple` such that we get
    /// `SlottedParameterMessagePath`
    fn to_slotted_parameter_message_path(self, configurables: &Self::Configurables) -> &Self::SlottedParameterMessagePath;
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

    /// A path parameter slot carrying only its type `Par`. Used for building stuff via the
    /// type system
    pub struct MessagePathSlotElement<Par: FeagiMessagePathParameterElement>(
        core::marker::PhantomData<Par>,
    );

    impl<Par: FeagiMessagePathParameterElement> MessagePathSlotElement<Par> {
        /// Create a parameter slot marker for type `Par`.
        #[inline]
        pub const fn new() -> Self {
            Self(core::marker::PhantomData)
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
            f.debug_struct("FeagiMessagePathParameterElement").finish()
        }
    }

    //endregion
    
}
