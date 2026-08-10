//! Coverage for the three wrapper families generated on top of the unified unsigned integer
//! trait: indexes, counts and plain unsigned data.
//!
//! The families are distinct types by construction, so a count cannot be passed where an index is
//! expected. That guarantee is enforced by the compiler and therefore cannot be asserted at
//! runtime; what is asserted here is that each family carries the surface it is supposed to, that
//! the role survives every conversion, and that reading the value out still works through deref.

use core::any::TypeId;
use feagi_data::values::quantizable::{
    QuantizedUnsignedIntegerTrait, UnsignedIntegerEnum, UnsignedIntegerQuantizationLevel, WrappedQuantizedCount, WrappedQuantizedCountEnum,
    WrappedQuantizedIndex, WrappedQuantizedIndexEnum, WrappedQuantizedUnsigned, WrappedQuantizedUnsignedEnum,
};
use feagi_data::{create_wrapped_quantized_count, create_wrapped_quantized_index, create_wrapped_quantized_unsigned};

create_wrapped_quantized_index!(
    /// A position within some imaginary collection, used to exercise the index family.
    pub TestNeuronIndex
);

create_wrapped_quantized_count!(
    /// How many elements that imaginary collection holds, used to exercise the count family.
    pub TestNeuronCount
);

create_wrapped_quantized_unsigned!(
    /// A plain unsigned datum that is neither a position nor a population.
    pub TestThreshold
);

//region Shape

#[test]
fn a_wrapper_costs_nothing_over_the_integer_it_holds() {
    assert_eq!(size_of::<TestNeuronIndex<u32>>(), size_of::<u32>());
    assert_eq!(size_of::<TestNeuronCount<u8>>(), size_of::<u8>());
    assert_eq!(size_of::<TestThreshold<u64>>(), size_of::<u64>());
}

#[test]
fn the_three_families_are_distinct_types_even_at_the_same_quantization() {
    assert_ne!(TypeId::of::<TestNeuronIndex<u32>>(), TypeId::of::<TestNeuronCount<u32>>());
    assert_ne!(TypeId::of::<TestNeuronCount<u32>>(), TypeId::of::<TestThreshold<u32>>());
    assert_ne!(TypeId::of::<TestThreshold<u32>>(), TypeId::of::<TestNeuronIndex<u32>>());
}

#[test]
fn a_wrapper_reports_the_level_of_the_integer_it_holds() {
    assert_eq!(TestNeuronIndex::<u16>::LEVEL, UnsignedIntegerQuantizationLevel::U16);
    assert_eq!(<TestNeuronCount<u32> as WrappedQuantizedCount>::LEVEL, UnsignedIntegerQuantizationLevel::U32);
    assert_eq!(<TestThreshold<u64> as WrappedQuantizedUnsigned>::LEVEL, UnsignedIntegerQuantizationLevel::U64);
}

//endregion

//region Construction and extraction

#[test]
fn a_wrapper_hands_back_exactly_what_was_put_in() {
    let index = TestNeuronIndex::<u32>::new(42);
    assert_eq!(index.deref(), 42u32);
    assert_eq!(*index.as_ref(), 42u32);
    assert_eq!(TestNeuronIndex::<u32>::from(42u32), index);
}

#[test]
fn a_wrapper_can_be_built_in_a_const_context() {
    const INDEX: TestNeuronIndex<u16> = TestNeuronIndex::const_new(7);
    assert_eq!(INDEX.const_deref(), 7u16);
}

#[test]
fn the_identity_constants_carry_the_wrapper_type() {
    assert_eq!(TestNeuronIndex::<u32>::QUANT_ZERO.deref(), 0u32);
    assert_eq!(TestNeuronIndex::<u32>::QUANT_ONE.deref(), 1u32);
    assert_eq!(TestNeuronCount::<u8>::QUANT_MAX.deref(), u8::MAX);
    assert_eq!(TestThreshold::<u32>::default().deref(), 0u32);
}

#[test]
fn building_from_a_usize_checks_bounds_when_asked_to() {
    assert_eq!(TestNeuronCount::<u8>::quant_try_from_usize(255).unwrap().deref(), 255u8);
    assert!(TestNeuronCount::<u8>::quant_try_from_usize(256).is_err());
    // The unchecked path is documented as truncating rather than failing.
    assert_eq!(TestNeuronCount::<u8>::quant_from_usize(256).deref(), 0u8);
}

//endregion

//region Deref

#[test]
fn reading_the_value_out_works_through_deref() {
    let index = TestNeuronIndex::<u32>::new(300);
    // quant_to_usize lives on QuantizedUnsignedIntegerTrait, reached by deref to the inner u32.
    assert_eq!(index.quant_to_usize(), 300);
    assert_eq!(index.quant_to_u16(), 300u16);
    assert_eq!(*index, 300u32);
}

#[test]
fn converting_to_another_quantization_through_deref_yields_a_bare_integer() {
    let count = TestNeuronCount::<u32>::new(200);
    let widened: u64 = count.to_quantization();
    assert_eq!(widened, 200u64);
    assert!(count.try_to_quantization::<u8>().is_ok());
    assert!(TestNeuronCount::<u32>::new(300).try_to_quantization::<u8>().is_err());
}

//endregion

//region Role preserving conversions

#[test]
fn building_from_another_quantization_keeps_the_wrapper_type() {
    let narrow: u8 = 200;
    let index: TestNeuronIndex<u64> = TestNeuronIndex::from_quantization(narrow);
    assert_eq!(index.deref(), 200u64);
}

#[test]
fn building_from_an_out_of_range_quantization_is_rejected() {
    let wide: u32 = 400;
    assert!(TestNeuronIndex::<u8>::try_from_quantization(wide).is_err());
    assert_eq!(TestNeuronIndex::<u8>::from_quantization_clamped(wide).deref(), u8::MAX);
}

#[test]
fn clamping_returns_the_wrapper_rather_than_a_bare_integer() {
    // This is the whole point of keeping clamp on the family trait instead of letting deref
    // supply it: the role must survive the clamp.
    let index = TestNeuronIndex::<u32>::new(1000);
    let clamped: TestNeuronIndex<u32> = index.clamp_for_quantization::<u8>();
    assert_eq!(clamped.deref(), u8::MAX as u32);

    let by_level: TestNeuronIndex<u32> = index.clamp_for_quantization_level_runtime(UnsignedIntegerQuantizationLevel::U8);
    assert_eq!(by_level.deref(), u8::MAX as u32);
}

//endregion

//region Arithmetic

#[test]
fn wrappers_add_and_subtract_within_their_own_family() {
    let a = TestNeuronIndex::<u32>::new(10);
    let b = TestNeuronIndex::<u32>::new(4);
    assert_eq!((a + b).deref(), 14u32);
    assert_eq!((a - b).deref(), 6u32);
    assert_eq!((a * b).deref(), 40u32);
    assert_eq!((a / b).deref(), 2u32);
    assert_eq!((a % b).deref(), 2u32);
}

#[test]
fn wrappers_support_compound_assignment() {
    let mut cursor = TestNeuronIndex::<u32>::QUANT_ZERO;
    cursor += TestNeuronIndex::QUANT_ONE;
    cursor += TestNeuronIndex::new(4);
    assert_eq!(cursor.deref(), 5u32);
    cursor -= TestNeuronIndex::QUANT_ONE;
    assert_eq!(cursor.deref(), 4u32);
    cursor *= TestNeuronIndex::new(3);
    assert_eq!(cursor.deref(), 12u32);
    cursor /= TestNeuronIndex::new(4);
    assert_eq!(cursor.deref(), 3u32);
    cursor %= TestNeuronIndex::new(2);
    assert_eq!(cursor.deref(), 1u32);
}

#[test]
fn wrappers_order_the_same_way_their_integers_do() {
    let mut values = [TestNeuronCount::<u16>::new(9), TestNeuronCount::new(2), TestNeuronCount::new(5)];
    values.sort();
    assert_eq!(values.map(|value| value.deref()), [2u16, 5, 9]);
}

//endregion

//region Generic use per family

/// Only accepts indexes. A count or a plain unsigned value would not satisfy the bound.
fn advance_by<I: WrappedQuantizedIndex>(index: I, step: I) -> I {
    index + step
}

/// Only accepts counts.
fn total_of<C: WrappedQuantizedCount>(left: C, right: C) -> C {
    left + right
}

/// Only accepts plain unsigned data.
fn halve<U: WrappedQuantizedUnsigned>(value: U) -> U {
    value / (U::QUANT_ONE + U::QUANT_ONE)
}

#[test]
fn each_family_trait_is_usable_as_a_generic_bound() {
    assert_eq!(advance_by(TestNeuronIndex::<u32>::new(10), TestNeuronIndex::new(5)).deref(), 15u32);
    assert_eq!(total_of(TestNeuronCount::<u16>::new(3), TestNeuronCount::new(4)).deref(), 7u16);
    assert_eq!(halve(TestThreshold::<u8>::new(9)).deref(), 4u8);
}

//endregion

//region Companion enums

#[test]
fn the_companion_enum_remembers_the_quantization_it_was_built_from() {
    assert_eq!(
        TestNeuronIndexEnum::new_from_quantized(TestNeuronIndex::<u8>::new(3)).get_level(),
        UnsignedIntegerQuantizationLevel::U8
    );
    assert_eq!(
        TestNeuronCountEnum::new_from_quantized(TestNeuronCount::<u32>::new(3)).get_level(),
        UnsignedIntegerQuantizationLevel::U32
    );
    assert_eq!(
        TestThresholdEnum::new_from_quantized(TestThreshold::<u64>::new(3)).get_level(),
        UnsignedIntegerQuantizationLevel::U64
    );
}

#[test]
fn the_companion_enum_round_trips_through_the_shared_value_enum() {
    let original = TestNeuronIndexEnum::new_from_quantized(TestNeuronIndex::<u16>::new(1234));
    let shared: UnsignedIntegerEnum = original.into_unsigned_integer_enum();
    assert_eq!(shared, UnsignedIntegerEnum::U16(1234));
    assert_eq!(TestNeuronIndexEnum::from_unsigned_integer_enum(shared), original);
}

#[test]
fn the_companion_enum_unwraps_back_into_a_wrapper_of_the_same_family() {
    let stored = TestNeuronCountEnum::new_from_quantized(TestNeuronCount::<u8>::new(200));
    assert_eq!(stored.to_usize(), 200);

    let widened: TestNeuronCount<u64> = stored.into_wrapped_quant();
    assert_eq!(widened.deref(), 200u64);

    let same_width: TestNeuronCount<u16> = stored.try_into_wrapped_quant().unwrap();
    assert_eq!(same_width.deref(), 200u16);
}

#[test]
fn the_companion_enum_refuses_a_quantization_that_cannot_hold_its_value() {
    let stored = TestThresholdEnum::new_from_quantized(TestThreshold::<u32>::new(70_000));
    assert!(stored.try_into_wrapped_quant::<u16>().is_err());
    assert!(stored.try_into_wrapped_quant::<u32>().is_ok());
    assert_eq!(stored.into_quant::<u64>(), 70_000u64);
}

//endregion
