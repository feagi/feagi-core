#[macro_export]
macro_rules! create_wrapped_index {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $quantization_level:ty
    ) => {
::paste::paste! {
        $(#[$meta])*
        $vis type $struct_name = [<$struct_name Generic>]< $quantization_level >;
        $crate::create_wrapped_quantized_index!($vis [<$struct_name Generic>]);
}
    }
}

// TODO uint wrapper!

#[macro_export]
macro_rules! create_wrapped_uint {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $quantization_level:ty
    ) => {
::paste::paste! {
        $(#[$meta])*
        $vis type $struct_name = [<$struct_name Generic>]< $quantization_level >;
        $crate::create_wrapped_quantized_index!($vis [<$struct_name Generic>]);
}
    }
}
