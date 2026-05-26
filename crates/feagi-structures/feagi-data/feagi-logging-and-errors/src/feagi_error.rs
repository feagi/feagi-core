pub use feagi_logging_and_errors_derive::{FeagiError, FeagiErrorKey};

/// A sized, no-alloc error key carrying a static context string and optional
/// typed fields that further identify the failing condition. Generics are
/// not supported! Best implemented with by deriving "FeagiErrorKey"
///
/// ```ignore
/// #[derive(FeagiErrorKey)]
/// pub struct InvalidNeuronId {
///     context: &'static str,
///     neuron_id: u32,
/// }
/// ```
pub trait FeagiErrorKeyTrait: core::fmt::Debug + core::fmt::Display + Sized + 'static {
    fn context(&self) -> &'static str;
}

/// A no-alloc FEAGI error enum.
///
/// Derived enums are expected to wrap either a `FeagiErrorKeyTrait` key or
/// another derived `FeagiErrorTrait` enum in each variant.
pub trait FeagiErrorTrait: core::error::Error + Sized + 'static {
    fn context(&self) -> &'static str;
}
