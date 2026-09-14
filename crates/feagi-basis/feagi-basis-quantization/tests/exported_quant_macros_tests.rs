use feagi_basis_quantization::values::quantizable::{
    DecimalQuantizationLevel, QuantizedDecimalTrait, QuantizedSignedIntegerTrait, QuantizedUnsignedIntegerTrait,
    QuantizedUnsignedPercentageTrait, SignedIntegerQuantizationLevel, UnsignedIntegerQuantizationLevel,
    WrappedQuantizedDecimalEnum,
};

feagi_basis_quantization::create_wrapped_quantized_unsigned_integer!(pub TestUnsignedWrap);
feagi_basis_quantization::create_wrapped_quantized_signed_integer!(pub TestSignedWrap);
feagi_basis_quantization::create_wrapped_quantized_decimal!(pub TestDecimalWrap);
feagi_basis_quantization::create_wrapped_percentage_unsigned!(pub TestPercentWrap);

#[test]
fn unsigned_wrapper_macro_supports_cross_quantization_and_runtime_clamping() {
    let wide = TestUnsignedWrap::<u16>::new(400);
    let narrowed = wide.clamp_for_quantization::<u8>();
    assert_eq!(narrowed.deref(), 255);

    let dynamic = TestUnsignedWrapEnum::new_from_quantized(TestUnsignedWrap::<u16>::new(123));
    assert_eq!(dynamic.get_level(), UnsignedIntegerQuantizationLevel::U16);
    assert_eq!(dynamic.try_into_quant::<u8>().unwrap(), 123u8);

    let out_of_range = TestUnsignedWrapEnum::new_from_quantized(TestUnsignedWrap::<u16>::new(800));
    assert!(out_of_range.try_into_quant::<u8>().is_err());
}

#[test]
fn signed_wrapper_macro_preserves_sign_and_reports_overflow_on_downcast() {
    let negative = TestSignedWrap::<i16>::new(-42);
    assert!(negative.is_negative());
    assert!(negative.is_zero_or_negative());

    let dynamic = TestSignedWrapEnum::new_from_quantized(negative);
    assert_eq!(dynamic.get_level(), SignedIntegerQuantizationLevel::I16);
    assert_eq!(dynamic.try_into_quant::<i8>().unwrap(), -42i8);

    let out_of_range = TestSignedWrapEnum::new_from_quantized(TestSignedWrap::<i16>::new(300));
    assert!(out_of_range.try_into_quant::<i8>().is_err());
}

#[test]
fn decimal_and_percentage_macros_compose_for_scaled_computation_and_dynamic_dispatch() {
    let value = TestDecimalWrap::<f32>::new(8.0);
    let quarter = TestPercentWrap::<f32>::new_checked(0.25).unwrap();
    let scaled = value.scale_by_unsigned_percentage(quarter.dewrap());
    assert!((scaled.deref() - 2.0).abs() < 1e-6);

    let dynamic = TestDecimalWrapEnum::new_from_quantized(TestDecimalWrap::<f32>::new(1.5));
    assert_eq!(dynamic.get_level(), DecimalQuantizationLevel::F32);
    assert!((dynamic.to_f64() - 1.5).abs() < 1e-9);

    let as_f64 = dynamic.into_wrapped_quant::<f64>();
    assert!((as_f64.deref() - 1.5).abs() < 1e-12);
}
