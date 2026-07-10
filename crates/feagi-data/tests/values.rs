//! Integration tests for the public surface of the `feagi_data::values` module.
//!
//! These exercise the exposed traits, concrete types, generated spatial structs and the
//! `#[macro_export]`ed wrapper macros that live under `values`.

use std::collections::HashSet;

use feagi_data::values::percentage::PercentageUnsigned;
use feagi_data::values::quantizable::custom_data_types::StorageF8;

use feagi_data::values::quantizable::{QuantizationLevel, QuantizedDecimalTrait, QuantizedElementBase, QuantizedIndexCountTrait, QuantizedSignedIntegerTrait, QuantizedUnsignedIntegerTrait};
use feagi_data::values::spatial::integer_signed::{
    SignedCoordinate2D, SignedCoordinate3D, SignedCoordinate4D,
};
use feagi_data::values::spatial::quantizable_index::{
    QuantizedIndexCoord2D, QuantizedIndexCoord3D, QuantizedIndexCoord4D,
    QuantizedIndexDimension2D, QuantizedIndexDimension3D,
};

// The wrapper macros are `#[macro_export]`ed, so they are reachable from the crate root.
// Instantiate concrete newtypes here so we can test what they generate.
feagi_data::create_wrapped_quantized_index!(
    /// Test newtype wrapping a quantized index/count value.
    pub TestWrappedIndex
);

feagi_data::create_wrapped_quantized_decimal!(
    /// Test newtype wrapping a quantized decimal value.
    pub TestWrappedDecimal
);

//region QuantizationLevel

#[test]
fn quantization_level_byte_discriminants() {
    assert_eq!(QuantizationLevel::Bit8 as u8, 1);
    assert_eq!(QuantizationLevel::Bit16 as u8, 2);
    assert_eq!(QuantizationLevel::Bit32 as u8, 4);
    assert_eq!(QuantizationLevel::Bit64 as u8, 8);
}

#[test]
fn quantization_level_minimum_for_usize() {
    use QuantizationLevel::*;
    assert_eq!(QuantizationLevel::minimum_quantization_needed_for_usize(0), Bit8);
    assert_eq!(QuantizationLevel::minimum_quantization_needed_for_usize(254), Bit8);
    // Boundaries use a strict `<`, so the max value of a level spills into the next one up.
    assert_eq!(QuantizationLevel::minimum_quantization_needed_for_usize(255), Bit16);
    assert_eq!(QuantizationLevel::minimum_quantization_needed_for_usize(65_534), Bit16);
    assert_eq!(QuantizationLevel::minimum_quantization_needed_for_usize(65_535), Bit32);
    assert_eq!(QuantizationLevel::minimum_quantization_needed_for_usize(1_000_000), Bit32);
    assert_eq!(
        QuantizationLevel::minimum_quantization_needed_for_usize(u32::MAX as usize),
        Bit64
    );
}

#[test]
fn quantization_level_derives() {
    let a = QuantizationLevel::Bit16;
    let b = a; // Copy
    assert_eq!(a, b);
    assert_eq!(a.clone(), QuantizationLevel::Bit16);
    assert_ne!(QuantizationLevel::Bit8, QuantizationLevel::Bit32);
    let mut set = HashSet::new();
    set.insert(QuantizationLevel::Bit64);
    assert!(set.contains(&QuantizationLevel::Bit64));
    assert!(format!("{:?}", a).contains("Bit16"));
}

//endregion

//region QuantizedIndexCountTrait

#[test]
fn quantized_index_count_constants() {
    assert_eq!(<u8 as QuantizedIndexCountTrait>::QUANT_ONE, 1);
    assert_eq!(<u8 as QuantizedIndexCountTrait>::QUANT_MAX, u8::MAX);
    assert_eq!(<u8 as QuantizedIndexCountTrait>::QUANT_MAX_AS_USIZE, u8::MAX as usize);
    assert_eq!(<u16 as QuantizedIndexCountTrait>::QUANT_MAX, u16::MAX);
    assert_eq!(<u32 as QuantizedIndexCountTrait>::QUANT_MAX, u32::MAX);
}

#[test]
fn quantized_index_count_u32_conversions() {
    assert_eq!(5u8.to_u32(), 5u32);
    assert_eq!(<u8 as QuantizedIndexCountTrait>::from_u32(5), 5u8);
    // from_u32 is unchecked: it simply truncates.
    assert_eq!(<u8 as QuantizedIndexCountTrait>::from_u32(300), 300u32 as u8);
    // from_u32_clamped saturates at the type's max.
    assert_eq!(<u8 as QuantizedIndexCountTrait>::from_u32_clamped(300), u8::MAX);
    assert_eq!(<u8 as QuantizedIndexCountTrait>::from_u32_clamped(100), 100u8);
    assert_eq!(<u16 as QuantizedIndexCountTrait>::from_u32_clamped(70_000), u16::MAX);
}


//endregion

//region QuantizedDecimalTrait

fn decimal_roundtrip<Q: QuantizedDecimalTrait>(value: f32) -> f32 {
    Q::from_f32(value).to_f32()
}

#[test]
fn quantized_decimal_roundtrip_exact_values() {
    // f32 is the identity implementation.
    assert_eq!(decimal_roundtrip::<f32>(1.5), 1.5);
    assert_eq!(decimal_roundtrip::<f32>(-3.25), -3.25);
    // f64 widens then narrows, exact for these values.
    assert_eq!(decimal_roundtrip::<f64>(2.25), 2.25);
    // StorageF8 can represent small powers of two exactly.
    assert_eq!(decimal_roundtrip::<StorageF8>(0.0), 0.0);
    assert_eq!(decimal_roundtrip::<StorageF8>(0.5), 0.5);
    assert_eq!(decimal_roundtrip::<StorageF8>(1.0), 1.0);
    assert_eq!(decimal_roundtrip::<StorageF8>(-2.0), -2.0);
}

//endregion

//region QuantizedSignedIntegerTrait

#[test]
fn quantized_signed_integer_predicates() {
    assert!((-3i32).is_negative());
    assert!(!(3i32).is_negative());
    assert!(!(0i32).is_negative());

    assert!((0i32).is_zero_or_negative());
    assert!((-1i32).is_zero_or_negative());
    assert!(!(1i32).is_zero_or_negative());

    assert!((-1i8).is_negative());
    assert!((-1i64).is_zero_or_negative());
    assert!((-1isize).is_negative());
}

//endregion

//region shared_traits: QuantizedElementBase / SupportsUintOps / SupportsBasicCoreMathOps

#[test]
fn quantized_element_base_constants() {
    assert_eq!(<u8 as QuantizedElementBase>::QUANTIZATION_LEVEL, QuantizationLevel::Bit8);
    assert_eq!(<u16 as QuantizedElementBase>::QUANTIZATION_LEVEL, QuantizationLevel::Bit16);
    assert_eq!(<u32 as QuantizedElementBase>::QUANTIZATION_LEVEL, QuantizationLevel::Bit32);
    assert_eq!(<u64 as QuantizedElementBase>::QUANTIZATION_LEVEL, QuantizationLevel::Bit64);
    assert_eq!(<f32 as QuantizedElementBase>::QUANTIZATION_LEVEL, QuantizationLevel::Bit32);

    assert_eq!(<u8 as QuantizedElementBase>::QUANT_ZERO, 0);
    assert_eq!(<i32 as QuantizedElementBase>::QUANT_ZERO, 0);
    assert_eq!(<f32 as QuantizedElementBase>::QUANT_ZERO, 0.0);
}

#[test]
fn supports_uint_ops() {
    assert_eq!(<u8>::QUANT_ONE, 1);
    assert_eq!(<u8>::QUANT_MAX, u8::MAX);
    assert_eq!(<u8>::QUANT_MAX_AS_USIZE, u8::MAX as usize);


    //endregion

    //region QuantizedUnsignedIntegerTrait (marker trait)
    
    //endregion

    //region StorageF8 custom data type

    #[test]
    fn storage_f8_zero_and_signs() {
        assert_eq!(StorageF8::ZERO.to_f32(), 0.0);
        assert!(!StorageF8::ZERO.is_negative());
        assert!(StorageF8::NEGATIVE_ZERO.is_negative());
        assert!(StorageF8::MAX.to_f32() > 0.0);
        assert!(StorageF8::MIN.to_f32() < 0.0);
        assert!(StorageF8::from_f32(-1.0).is_negative());
        assert!(!StorageF8::from_f32(1.0).is_negative());
    }

    #[test]
    fn storage_f8_bit_roundtrip() {
        let bits = StorageF8::from_f32(1.0).to_bits();
        assert_eq!(StorageF8::from_bits(bits).to_bits(), bits);
        assert_eq!(StorageF8::from_bits(bits).to_f32(), 1.0);
    }

    #[test]
    fn storage_f8_conversions_and_math() {
        let one: StorageF8 = 1.0f32.into();
        let two = StorageF8::from_f32(2.0);
        assert_eq!(one + one, two);
        assert_eq!(two - one, one);
        assert_eq!(two * one, two);
        assert_eq!(two / two, one);

        let mut acc = one;
        acc += one;
        assert_eq!(acc, two);
        acc -= one;
        assert_eq!(acc, one);

        let back: f32 = two.into();
        assert_eq!(back, 2.0);
        assert!(one < two);
    }

    //endregion

    //region PercentageUnsigned

    #[test]
    fn percentage_unsigned_clamping_and_roundtrip() {
        assert_eq!(PercentageUnsigned::from_f32_clamped(1.0).to_f32_0_1(), 1.0);
        // Out of range inputs are clamped to [0, 1].
        assert_eq!(PercentageUnsigned::from_f32_clamped(2.0).to_f32_0_1(), 1.0);
        assert_eq!(PercentageUnsigned::from_f32_clamped(-1.0).to_f32_0_1(), 0.0);

        let half = PercentageUnsigned::from_f32_clamped(0.5).to_f32_0_1();
        assert!((half - 0.5).abs() < 0.01, "got {half}");
    }

    #[test]
    fn percentage_unsigned_default_and_ordering() {
        assert_eq!(PercentageUnsigned::default(), PercentageUnsigned::from_f32_clamped(0.0));
        let low = PercentageUnsigned::from_f32_clamped(0.2);
        let high = PercentageUnsigned::from_f32_clamped(0.8);
        assert!(low < high);

        let mut set = HashSet::new();
        set.insert(high.clone());
        assert!(set.contains(&high));
        assert!(format!("{:?}", low).contains("PercentageUnsigned"));
    }

    //endregion

    //region Spatial: generated quantizable coordinate / dimension structs

    #[test]
    fn quantized_index_coord_new_and_getters() {
        let mut coord = QuantizedIndexCoord3D::<u32>::new(1, 2, 3);
        assert_eq!(*coord.get_x(), 1);
        assert_eq!(*coord.get_y(), 2);
        assert_eq!(*coord.get_z(), 3);

        *coord.get_x_mut() = 10;
        *coord.get_z_mut() = 30;
        assert_eq!(*coord.get_x(), 10);
        assert_eq!(*coord.get_z(), 30);

        let coord4 = QuantizedIndexCoord4D::<u16>::new(1, 2, 3, 4);
        assert_eq!(*coord4.get_w(), 4);
    }

    #[test]
    fn quantized_index_coord_derived_traits() {
        let a = QuantizedIndexCoord2D::<u16>::new(3, 4);
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, QuantizedIndexCoord2D::<u16>::new(3, 5));

        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));

        assert!(!format!("{:?}", a).is_empty());
    }

    #[test]
    fn quantized_index_dimension_element_count() {
        let dims2 = QuantizedIndexDimension2D::<u32>::new(3, 4);
        assert_eq!(dims2.number_contained_elements(), 12);

        let dims3 = QuantizedIndexDimension3D::<u32>::new(4, 5, 6);
        assert_eq!(dims3.number_contained_elements(), 120);
        assert_eq!(*dims3.get_x(), 4);
        assert_eq!(*dims3.get_y(), 5);
        assert_eq!(*dims3.get_z(), 6);
    }

    #[test]
    fn quantized_index_dimension_linear_index_roundtrip() {
        let dims = QuantizedIndexDimension3D::<u32>::new(4, 5, 6);
        let coord = QuantizedIndexCoord3D::<u32>::new(1, 2, 3);

        // x increments fastest: 1 + 2*4 + 3*(4*5) = 69.
        let linear = dims.coordinate_to_linear_index(coord.clone());
        assert_eq!(linear, 69);

        let back = dims.linear_to_coordinate_index(linear);
        assert_eq!(back, coord);

        // Origin maps to 0 and back.
        let origin = QuantizedIndexCoord3D::<u32>::new(0, 0, 0);
        assert_eq!(dims.coordinate_to_linear_index(origin.clone()), 0);
        assert_eq!(dims.linear_to_coordinate_index(0), origin);
    }

    //endregion

    //region Spatial: signed integer coordinates

    #[test]
    fn signed_coordinates_new_and_getters() {
        let mut c2 = SignedCoordinate2D::new(-1, 2);
        assert_eq!(*c2.get_x(), -1);
        assert_eq!(*c2.get_y(), 2);
        *c2.get_x_mut() = 7;
        assert_eq!(*c2.get_x(), 7);

        let c3 = SignedCoordinate3D::new(-1, 2, -3);
        assert_eq!(*c3.get_z(), -3);

        let mut c4 = SignedCoordinate4D::new(1, -2, 3, -4);
        assert_eq!(*c4.get_w(), -4);
        *c4.get_w_mut() = 9;
        assert_eq!(*c4.get_w(), 9);
    }

    //endregion

    //region Exported wrapper macros

    #[test]
    fn create_wrapped_quantized_index_macro() {
        let a: TestWrappedIndex<u32> = 10u32.into();
        let b: TestWrappedIndex<u32> = 4u32.into();

        assert_eq!(*a.as_ref(), 10);
        assert_eq!(*(a + b).as_ref(), 14);
        assert_eq!(*(a - b).as_ref(), 6);
        assert_eq!(*(a * b).as_ref(), 40);
        assert_eq!(*(a / b).as_ref(), 2);
        assert_eq!(*(a % b).as_ref(), 2);

        let mut c = a;
        c += b;
        assert_eq!(*c.as_ref(), 14);
        c -= b;
        assert_eq!(*c.as_ref(), 10);
        c *= b;
        assert_eq!(*c.as_ref(), 40);
        c /= b;
        assert_eq!(*c.as_ref(), 10);
        c %= b;
        assert_eq!(*c.as_ref(), 2);

        // Derived ordering / hashing / equality.
        assert!(b < a);
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&a));

        // Zero-cost reference conversions (`From<&Q> for &Self`).
        let shared: &TestWrappedIndex<u32> = (&7u32).into();
        assert_eq!(*shared.as_ref(), 7);

        // Mutable reference conversion writes straight through to the backing value.
        let mut backing = 9u32;
        {
            let exclusive: &mut TestWrappedIndex<u32> = (&mut backing).into();
            *exclusive.as_mut() = 21;
        }
        assert_eq!(backing, 21);
    }

    #[test]
    fn create_wrapped_quantized_decimal_macro() {
        let a: TestWrappedDecimal<f32> = 1.5f32.into();
        let b: TestWrappedDecimal<f32> = 0.5f32.into();

        assert_eq!(*a.as_ref(), 1.5);
        assert_eq!(*(a + b).as_ref(), 2.0);
        assert_eq!(*(a - b).as_ref(), 1.0);
        assert_eq!(*(a * b).as_ref(), 0.75);
        assert_eq!(*(a / b).as_ref(), 3.0);

        let mut c = a;
        c += b;
        assert_eq!(*c.as_ref(), 2.0);

        assert!(b < a);

        let shared: &TestWrappedDecimal<f32> = (&2.5f32).into();
        assert_eq!(*shared.as_ref(), 2.5);
    }
}

//endregion
