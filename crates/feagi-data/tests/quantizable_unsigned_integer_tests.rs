//! Coverage for the single quantized unsigned integer family that indexes, counts and plain
//! unsigned data all share.

use feagi_data::values::quantizable::{
    QuantizationLevelPacking, QuantizedElementBase, QuantizedUnsignedIntegerTrait, UnsignedIntegerEnum, UnsignedIntegerQuantizationLevel,
};

#[test]
fn every_primitive_reports_its_own_level() {
    assert_eq!(<u8 as QuantizedUnsignedIntegerTrait>::LEVEL, UnsignedIntegerQuantizationLevel::U8);
    assert_eq!(<u16 as QuantizedUnsignedIntegerTrait>::LEVEL, UnsignedIntegerQuantizationLevel::U16);
    assert_eq!(<u32 as QuantizedUnsignedIntegerTrait>::LEVEL, UnsignedIntegerQuantizationLevel::U32);
    assert_eq!(<u64 as QuantizedUnsignedIntegerTrait>::LEVEL, UnsignedIntegerQuantizationLevel::U64);
}

#[test]
fn quantization_levels_round_trip_through_a_byte() {
    for level in [
        UnsignedIntegerQuantizationLevel::U8,
        UnsignedIntegerQuantizationLevel::U16,
        UnsignedIntegerQuantizationLevel::U32,
        UnsignedIntegerQuantizationLevel::U64,
        UnsignedIntegerQuantizationLevel::Usize,
    ] {
        let byte: u8 = level.into();
        assert_eq!(UnsignedIntegerQuantizationLevel::try_from(byte), Ok(level));
    }
}

#[test]
fn an_unknown_byte_is_not_a_quantization_level() {
    assert_eq!(UnsignedIntegerQuantizationLevel::try_from(5u8), Err(()));
    assert_eq!(UnsignedIntegerQuantizationLevel::try_from(u8::MAX), Err(()));
}

#[test]
fn every_level_fits_the_bits_reserved_for_packing() {
    // The packing reserves three bits, so every discriminant has to stay under eight.
    let widest: u8 = UnsignedIntegerQuantizationLevel::Usize.into();
    assert!((widest as usize) < (1usize << <UnsignedIntegerQuantizationLevel as QuantizationLevelPacking>::NUMBER_BITS));
}

#[test]
fn zero_and_one_are_the_quantized_identities() {
    assert_eq!(<u8 as QuantizedElementBase>::QUANT_ZERO, 0u8);
    assert_eq!(<u32 as QuantizedElementBase>::QUANT_ONE, 1u32);
    assert_eq!(<u64 as QuantizedUnsignedIntegerTrait>::QUANT_MAX, u64::MAX);
}

#[test]
fn widening_a_quantization_keeps_the_value() {
    let narrow: u8 = 200;
    let widened: u64 = narrow.to_quantization();
    assert_eq!(widened, 200u64);
    assert_eq!(narrow.quant_to_usize(), 200);
}

#[test]
fn narrowing_within_range_keeps_the_value() {
    let wide: u32 = 250;
    assert_eq!(wide.try_to_quantization::<u8>().unwrap(), 250u8);
}

#[test]
fn narrowing_out_of_range_is_rejected_rather_than_truncated() {
    let wide: u32 = 300;
    assert!(wide.try_to_quantization::<u8>().is_err());
}

#[test]
fn narrowing_out_of_range_saturates_when_clamping_is_asked_for() {
    let wide: u32 = 300;
    assert_eq!(wide.to_quantization_clamped::<u8>(), u8::MAX);
}

#[test]
fn clamping_for_a_quantization_does_not_change_the_quantization() {
    // The value is squeezed into what a u8 could hold, but stays a u32.
    let wide: u32 = 1000;
    let clamped: u32 = wide.clamp_for_quantization::<u8>();
    assert_eq!(clamped, u8::MAX as u32);
}

#[test]
fn clamping_for_a_runtime_level_matches_the_compile_time_clamp() {
    let wide: u64 = 100_000;
    assert_eq!(
        wide.clamp_for_quantization_level_runtime(UnsignedIntegerQuantizationLevel::U16),
        u16::MAX as u64
    );
    assert_eq!(wide.clamp_for_quantization_level_runtime(UnsignedIntegerQuantizationLevel::U32), 100_000u64);
}

#[test]
fn converting_from_usize_checks_bounds_when_asked_to() {
    assert_eq!(<u8 as QuantizedUnsignedIntegerTrait>::quant_try_from_usize(255).unwrap(), 255u8);
    assert!(<u8 as QuantizedUnsignedIntegerTrait>::quant_try_from_usize(256).is_err());
    assert_eq!(<u16 as QuantizedUnsignedIntegerTrait>::quant_try_from_usize(65_535).unwrap(), u16::MAX);
    assert!(<u16 as QuantizedUnsignedIntegerTrait>::quant_try_from_usize(65_536).is_err());
}

#[test]
fn the_enum_remembers_which_quantization_it_came_from() {
    assert_eq!(UnsignedIntegerEnum::new_from_quantized(7u8).get_level(), UnsignedIntegerQuantizationLevel::U8);
    assert_eq!(UnsignedIntegerEnum::new_from_quantized(7u16).get_level(), UnsignedIntegerQuantizationLevel::U16);
    assert_eq!(UnsignedIntegerEnum::new_from_quantized(7u32).get_level(), UnsignedIntegerQuantizationLevel::U32);
    assert_eq!(UnsignedIntegerEnum::new_from_quantized(7u64).get_level(), UnsignedIntegerQuantizationLevel::U64);
}

#[test]
fn the_enum_round_trips_a_value_through_a_wider_quantization() {
    let stored = UnsignedIntegerEnum::new_from_quantized(200u8);
    assert_eq!(stored.to_usize(), 200);
    assert_eq!(stored.into_quant::<u64>(), 200u64);
    assert_eq!(stored.try_into_quant::<u16>().unwrap(), 200u16);
}

#[test]
fn the_enum_refuses_a_quantization_that_cannot_hold_its_value() {
    let stored = UnsignedIntegerEnum::new_from_quantized(70_000u32);
    assert!(stored.try_into_quant::<u16>().is_err());
    assert!(stored.try_into_quant::<u32>().is_ok());
}
