pub use feagi_logging_and_errors_derive::{FeagiError, FeagiErrorKey};

// TODO print error as a feagi log?

#[macro_export]
macro_rules! generate_feagi_error {
    (
        $(#[doc = $doc:expr])?
        $error_name:ident,
        keys: {
            $( $key_name:ident : $key_type:ty ),* $(,)?
        },
        sub_errors:
        {
            $( $sub_error_name:ident : $sub_error_type:ty ),* $(,)?
        }$(,)?
    ) => {
        $(#[doc = $doc])?
        #[derive(FeagiError)]
        pub enum $error_name {
            $( $key_name($key_type) ),*
            $( $sub_error_name($sub_error_type) ),*
        }

        impl $error_name
        {
            ::paste::paste!
            {
                $(
                pub const fn [<const_from_ $key_name:snake>](key: $key_type)-> $error_name {
                    $error_name::$key_name(key)
                }
                )*
            }
        }

        $(
        impl Into<$error_name> for $key_type {
            fn into(self) -> $error_name {
                $error_name::$key_name(self)
            }
        }
        )*


        $(
        impl Into<$error_name> for $sub_error_type {
            fn into(self) -> $error_name {
                $error_name::$sub_error_name(self)
            }
        }
        )*
    };


}

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
