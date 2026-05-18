
/// Generate a struct of given name with multiple parallel vector members of given types
#[macro_export]
macro_rules! generate_linear_parallel_vector_struct {
    (
        $(#[doc = $doc:expr])?
        $name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        $(#[doc = $doc:expr])?
        pub struct $name {
            $(
                $field: Vec<$ty>,
            )*
        }
    };
}

/// Generate a struct of given name with multiple parallel array members of given types
#[macro_export]
macro_rules! generate_linear_parallel_array_struct {
    (
        $(#[doc = $doc:expr])?
        $name:ident, $number_elements:expr,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        $(#[doc = $doc:expr])?
        pub struct $name {
            $(
                $field: [$ty: $number_elements:expr],
            )*
        }
    };
}

/// Generate a trait that implements parallel immutable slice access
#[macro_export]
macro_rules! generate_linear_parallel_access_trait_immut {
    (
        $(#[doc = $doc:expr])?
        $name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {
            $(#[doc = $doc:expr])?
            pub trait [<$name> ParallelAccess] {
                $(
                    crate::define_ref_immut_access_trait_methods!($field, $ty, $field,)
                )*
            }
        }
    };
}

/// Generate a trait that implements parallel mutable slice access and extends a trait that 
/// adds parallel immutable slice access
#[macro_export]
macro_rules! generate_linear_parallel_access_trait_immut_mut {
    (
        $(#[doc_mut = $doc_mut:expr])?
        $name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
        $(#[doc_immut = $doc_immut:expr])?
    ) => {
        ::paste::paste! {
            $(#[doc_mut = $doc_mut:expr])?
            pub trait [<$name> ParallelAccess] {
                $(
                    crate::define_ref_immut_access_trait_methods!($field, $ty, $field,)
                )*
            }

            $(#[doc_immut = $doc_immut:expr])?
            pub trait [<$name> ParallelAccessMut]: [<$name> ParallelAccess] {
                $(
                    crate::define_ref_mut_access_trait_methods!($field, $ty, $field,)
                )*
            }
        }
    };
}

