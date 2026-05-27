use feagi_data::{
    create_quantized_decimal_wrapper, create_quantized_index_count_wrapper,
    create_quantized_signed_integer_wrapper, create_quantized_unsigned_integer_wrapper,
};
use feagi_data::quantizable_linear::base_types::{
    QuantizedDecimalTrait, QuantizedIndexCountTrait, QuantizedSignedIntegerTrait,
};
use feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;

create_quantized_index_count_wrapper!(PublicIndexAxis);
create_quantized_index_count_wrapper!(pub(crate) CrateIndexAxis);
create_quantized_index_count_wrapper!(private PrivateIndexAxis);
create_quantized_index_count_wrapper!(ConcreteIndexAxis, u32);

create_quantized_unsigned_integer_wrapper!(PublicUnsignedAxis);
create_quantized_unsigned_integer_wrapper!(pub(crate) CrateUnsignedAxis);
create_quantized_unsigned_integer_wrapper!(private PrivateUnsignedAxis);
create_quantized_unsigned_integer_wrapper!(ConcreteUnsignedAxis, u32);

create_quantized_signed_integer_wrapper!(PublicSignedAxis);
create_quantized_signed_integer_wrapper!(pub(crate) CrateSignedAxis);
create_quantized_signed_integer_wrapper!(private PrivateSignedAxis);
create_quantized_signed_integer_wrapper!(ConcreteSignedAxis, i32);

create_quantized_decimal_wrapper!(PublicDecimalAxis);
create_quantized_decimal_wrapper!(pub(crate) CrateDecimalAxis);
create_quantized_decimal_wrapper!(private PrivateDecimalAxis);
create_quantized_decimal_wrapper!(ConcreteDecimalAxis, f32);

#[test]
fn generated_index_count_wrappers_support_visibility_forms_and_index_methods() {
    let public = PublicIndexAxis::<u16>::from_u32(7);
    let crate_visible = CrateIndexAxis::<u16>::from_u32_clamped(9);
    let mut private = PrivateIndexAxis::<u8>::from_u32(2);

    private += PrivateIndexAxis::from_u32(3);

    assert_eq!(public.to_u32(), 7);
    assert_eq!(crate_visible.to_u32(), 9);
    assert_eq!(private.to_u32(), 5);
}

#[test]
fn generated_unsigned_wrappers_support_uint_math_and_visibility_forms() {
    let public = PublicUnsignedAxis::<u16>::wrap(10);
    let crate_visible = CrateUnsignedAxis::<u16>::wrap(4);
    let mut private = PrivateUnsignedAxis::<u16>::wrap(9);

    private %= PrivateUnsignedAxis::wrap(4);

    assert_eq!((public % PublicUnsignedAxis::wrap(4)).unwrap(), 2);
    assert_eq!((crate_visible + CrateUnsignedAxis::wrap(3)).unwrap(), 7);
    assert_eq!(private.unwrap(), 1);
}

#[test]
fn generated_signed_wrappers_forward_signed_integer_methods() {
    let public = PublicSignedAxis::<i16>::wrap(-3);
    let crate_visible = CrateSignedAxis::<i16>::wrap(0);
    let private = PrivateSignedAxis::<i16>::wrap(5);

    assert!(public.is_negative());
    assert!(crate_visible.is_zero_or_negative());
    assert!(!private.is_negative());
}

#[test]
fn generated_decimal_wrappers_forward_decimal_methods() {
    let public = PublicDecimalAxis::<f32>::from_f32(1.5);
    let crate_visible = CrateDecimalAxis::<f32>::from_f32(2.0);
    let mut private = PrivateDecimalAxis::<f32>::from_f32(0.0);

    private.load_f32_inplace(3.25);

    assert_eq!(public.to_f32(), 1.5);
    assert_eq!((crate_visible + CrateDecimalAxis::from_f32(0.5)).to_f32(), 2.5);
    assert_eq!(private.to_f32(), 3.25);
}

#[test]
fn generated_concrete_wrappers_do_not_require_generic_quantization_parameters() {
    let index = ConcreteIndexAxis::from_u32(12);
    let unsigned = ConcreteUnsignedAxis::wrap(10);
    let signed = ConcreteSignedAxis::wrap(-4);
    let decimal = ConcreteDecimalAxis::from_f32(1.25);

    assert_eq!(index.const_take(), 12);
    assert_eq!((unsigned % ConcreteUnsignedAxis::wrap(3)).const_take(), 1);
    assert!(signed.is_negative());
    assert_eq!(decimal.const_take(), 1.25);
}
