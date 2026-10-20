
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

    /// The flat tuple representation of the path where &str is the defined parts and 
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

    /// Something that can be serialized to become part of the message path dynamically as
    /// a parameter.
    pub trait FeagiMessagePathParameterElement: core::str::FromStr + core::fmt::Debug 
    + serde::Serialize + serde::de::DeserializeOwned
    {
        fn to_str_slice(&self) -> &str;
    }

    #[cfg(feature = "alloc")]
    impl FeagiMessagePathParameterElement for String {
        fn to_str_slice(&self) -> &str {
            self.as_str()
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
