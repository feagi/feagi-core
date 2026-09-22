use core::ops::Range;

use feagi_basis_collections::bit_packed_bool::bit_batch_word::{BitBatchWord, BitBatchWordSize};
use feagi_basis_collections::bit_packed_bool::par_data::{
    BitBatchParDataArray, BitBatchParDataSlice, BitBatchParDataSliceMut,
};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;

//region BitBatchWord

#[test]
fn bit_batch_word_u8_constants_and_size_enum() {
    assert_eq!(u8::NUMBER_BITS, 8);
    assert_eq!(u8::NUMBER_BYTES, 1);
    assert_eq!(u8::BIT_SHIFT_DISTANCE, 3);
    assert_eq!(u8::BIT_BATCH_WORD_SIZE.get_as_number_bytes(), 1);
    assert_eq!(
        BitBatchWordSize::get_size_from_byte_count(1).get_as_number_bytes(),
        1
    );
    assert_eq!(BitBatchWordSize::U8.get_as_number_bytes(), 1);
}

#[test]
fn bit_batch_word_u16_u32_u64_constants() {
    assert_eq!(u16::NUMBER_BITS, 16);
    assert_eq!(u16::BIT_SHIFT_DISTANCE, 4);
    assert_eq!(u16::BIT_BATCH_WORD_SIZE.get_as_number_bytes(), 2);

    assert_eq!(u32::NUMBER_BITS, 32);
    assert_eq!(u32::BIT_SHIFT_DISTANCE, 5);
    assert_eq!(u32::BIT_BATCH_WORD_SIZE.get_as_number_bytes(), 4);

    assert_eq!(u64::NUMBER_BITS, 64);
    assert_eq!(u64::BIT_SHIFT_DISTANCE, 6);
    assert_eq!(u64::BIT_BATCH_WORD_SIZE.get_as_number_bytes(), 8);
}

#[test]
fn bit_batch_word_size_maps_all_supported_byte_counts() {
    for bytes in [1u8, 2, 4, 8] {
        assert_eq!(
            BitBatchWordSize::get_size_from_byte_count(bytes).get_as_number_bytes(),
            bytes
        );
    }
}

#[test]
fn u32_get_set_bit_and_count_bits() {
    let mut word = u32::from_usize(0);
    assert_eq!(word.count_bits(), 0);

    for index in [0usize, 5, 31] {
        assert!(!word.get_bit(index));
        word.set_bit(index, true);
        assert!(word.get_bit(index));
        assert_eq!(word.count_bits(), 1);
        word.set_bit(index, false);
        assert!(!word.get_bit(index));
        assert_eq!(word.count_bits(), 0);
    }

    word.set_bit(0, true);
    word.set_bit(1, true);
    word.set_bit(2, true);
    assert_eq!(word.count_bits(), 3);
    assert_eq!(word.as_usize(), 0b111);
}

#[test]
fn u64_from_usize_roundtrip_preserves_low_bits() {
    let value = 0xDEAD_BEEF_u64;
    let word = u64::from_usize(value as usize);
    assert_eq!(word.as_usize(), value as usize);
    assert_eq!(word.count_bits(), value.count_ones());
}

//endregion

//region BitBatchParData

feagi_basis_quantization::create_wrapped_quantized_unsigned_integer!(pub TestQuant);

type WordIdx = TestQuant<u8>;
type BoolIdx = TestQuant<u8>;

fn quant_usize(value: usize) -> BoolIdx {
    BoolIdx::quant_from_usize_unchecked(value)
}

fn quant_word(value: usize) -> WordIdx {
    WordIdx::quant_from_usize_unchecked(value)
}

fn assert_word_subslice(
    view: &BitBatchParDataSlice<'_, WordIdx, BoolIdx, u32>,
    expected_words: &[u32],
    expected_valid_bools: usize,
) {
    assert_eq!(view.len().quant_to_usize(), expected_words.len());
    assert_eq!(
        view.number_valid_bools().quant_to_usize(),
        expected_valid_bools
    );
    for (i, expected) in expected_words.iter().enumerate() {
        assert_eq!(view.get_word(quant_word(i)), Some(expected));
    }
}

//region Array backend

#[test]
fn array_backend_from_array_and_new_uniform_store_words() {
    let mut words = [0u32, 0u32];
    words[0].set_bit(3, true);
    words[1].set_bit(10, true);

    let from = BitBatchParDataArray::<WordIdx, BoolIdx, u32, 2>::from_array(words, quant_usize(42));
    assert_eq!(from.number_valid_bools(), quant_usize(42));
    assert_eq!(from.len().quant_to_usize(), 2);
    assert_eq!(from.get_bool(quant_usize(3)), Some(true));
    assert_eq!(from.get_bool(quant_usize(32 + 10)), Some(true));
    assert_eq!(from.into_array(), words);

    let uniform = BitBatchParDataArray::<WordIdx, BoolIdx, u32, 2>::new_uniform(0xA5A5_A5A5, quant_usize(50));
    assert_eq!(uniform.store(), &[0xA5A5_A5A5, 0xA5A5_A5A5]);
}

#[test]
fn array_backend_subslice_words_valid_bools_and_bool_access() {
    let mut words = [0u32, 0u32, 0u32];
    words[1].set_bit(0, true);
    words[1].set_bit(5, true);
    let data = BitBatchParDataArray::<WordIdx, BoolIdx, u32, 3>::from_array(words, quant_usize(70));

    let range = Range {
        start: quant_word(1),
        end: quant_word(3),
    };
    let view = data.subslice(range).expect("array subslice");
    assert_word_subslice(&view, &[words[1], words[2]], 38);
    assert_eq!(view.get_bool(quant_usize(0)), Some(true));
    assert_eq!(view.get_bool(quant_usize(5)), Some(true));
    assert_eq!(view.get_bool(quant_usize(39)), None);
}

#[test]
fn array_backend_subslice_mut_updates_parent_words() {
    let words = [1u32, 2u32, 3u32];
    let mut data =
        BitBatchParDataArray::<WordIdx, BoolIdx, u32, 3>::from_array(words, quant_usize(90));
    let expected_valid = data.number_valid_bools_for_word_range(1, 3).quant_to_usize();
    let mut view = data
        .subslice_mut(Range {
            start: quant_word(1),
            end: quant_word(3),
        })
        .expect("array subslice_mut");
    assert_eq!(view.iter_words().copied().collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(view.number_valid_bools().quant_to_usize(), expected_valid);
    assert_eq!(view.set_word(quant_word(0), 0xBEEF), Some(2));
    assert_eq!(data.into_array(), [1, 0xBEEF, 3]);
}

//endregion

//region Shared slice backend

#[test]
fn shared_slice_backend_new_and_as_words_slice() {
    let words = [7u32, 8u32];
    let slice_view = BitBatchParDataSlice::<WordIdx, BoolIdx, u32>::new(&words, quant_usize(40));
    assert_word_subslice(&slice_view, &[7, 8], 40);
    assert_eq!(slice_view.into_slice(), &words);

    let data = BitBatchParDataArray::<WordIdx, BoolIdx, u32, 2>::from_array(words, quant_usize(40));
    let borrowed = data.as_words_slice();
    assert_word_subslice(&borrowed, &[7, 8], 40);
}

#[test]
fn shared_slice_backend_subslice_from_slice_parent() {
    let words = [10u32, 20u32, 30u32];
    let parent = BitBatchParDataSlice::<WordIdx, BoolIdx, u32>::new(&words, quant_usize(80));
    let view = parent
        .subslice(Range {
            start: quant_word(1),
            end: quant_word(3),
        })
        .expect("slice subslice");
    assert_word_subslice(&view, &[20, 30], 48);
}

//endregion

//region Mutable slice backend

#[test]
fn mutable_slice_backend_new_reborrow_and_bool_mutation() {
    let mut words = [0u32; 2];
    let mut data = BitBatchParDataSliceMut::<WordIdx, BoolIdx, u32>::new(&mut words, quant_usize(40));

    assert_eq!(data.set_word(quant_word(0), 0xFFFF_FFFF), Some(0));
    data.get_word_from_bool_index_mut(quant_usize(5))
        .expect("bool in range")
        .set_bit(5, true);
    assert_eq!(data.get_bool(quant_usize(5)), Some(true));

    let reborrowed = data.reborrow();
    assert_eq!(reborrowed.len().quant_to_usize(), 2);
    assert_eq!(words[0], 0xFFFF_FFFF);
}

#[test]
fn mutable_slice_backend_subslice_mut_preserves_word_and_valid_bool_data() {
    let mut words = [1u32, 2u32, 3u32];
    let mut data = BitBatchParDataSliceMut::<WordIdx, BoolIdx, u32>::new(&mut words, quant_usize(65));
    let mut sub = data
        .subslice_mut(Range {
            start: quant_word(1),
            end: quant_word(3),
        })
        .expect("mut slice subslice");
    assert_eq!(sub.iter_words().copied().collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(sub.number_valid_bools().quant_to_usize(), 33);
    assert_eq!(sub.set_word(quant_word(1), 0x1234), Some(3));
    assert_eq!(words, [1, 2, 0x1234]);
}

//endregion

#[test]
fn get_word_bit_index_uses_lower_mask_for_u32_words() {
    let data = BitBatchParDataArray::<WordIdx, BoolIdx, u32, 1>::new_uniform(0, quant_usize(1));
    assert_eq!(data.get_word_bit_index_from_bool_index(quant_usize(17)), 17);
    assert_eq!(data.get_word_bit_index_from_bool_index(quant_usize(31)), 31);
}

#[test]
fn word_length_with_padding_matches_ceil_div_for_u32_words() {
    assert_eq!(
        BitBatchParDataArray::<WordIdx, BoolIdx, u32, 1>::word_length_with_padding(quant_usize(0))
            .quant_to_usize(),
        0
    );
    assert_eq!(
        BitBatchParDataArray::<WordIdx, BoolIdx, u32, 1>::word_length_with_padding(quant_usize(32))
            .quant_to_usize(),
        1
    );
    assert_eq!(
        BitBatchParDataArray::<WordIdx, BoolIdx, u32, 1>::word_length_with_padding(quant_usize(33))
            .quant_to_usize(),
        2
    );
}

#[test]
fn number_valid_bools_for_word_range_clamps_to_collection_bounds() {
    let data = BitBatchParDataArray::<WordIdx, BoolIdx, u32, 3>::new_uniform(0, quant_usize(70));
    assert_eq!(
        data.number_valid_bools_for_word_range(0, 1).quant_to_usize(),
        32
    );
    assert_eq!(
        data.number_valid_bools_for_word_range(1, 3).quant_to_usize(),
        38
    );
    assert_eq!(
        data.number_valid_bools_for_word_range(2, 3).quant_to_usize(),
        6
    );
}

#[cfg(feature = "alloc")]
mod vector_backend {
    use super::*;
    use feagi_basis_collections::bit_packed_bool::par_data::BitBatchParDataVector;

    #[test]
    fn vector_backend_new_uniform_from_vec_and_into_vec() {
        let uniform = BitBatchParDataVector::<WordIdx, BoolIdx, u32>::new_uniform(quant_usize(33), 0);
        assert_eq!(uniform.len().quant_to_usize(), 2);
        assert_eq!(uniform.number_valid_bools(), quant_usize(33));
        assert_eq!(uniform.into_vec(), vec![0, 0]);

        let wrapped = BitBatchParDataVector::<WordIdx, BoolIdx, u32>::from_vec(vec![1, 2, 3], quant_usize(80));
        assert_eq!(wrapped.store(), &vec![1u32, 2, 3][..]);
        assert_eq!(wrapped.into_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn vector_backend_subslice_words_and_valid_bools() {
        let wrapped =
            BitBatchParDataVector::<WordIdx, BoolIdx, u32>::from_vec(vec![10, 20, 30, 40], quant_usize(100));
        let view = wrapped
            .subslice(Range {
                start: quant_word(1),
                end: quant_word(3),
            })
            .expect("vector subslice");
        assert_word_subslice(&view, &[20, 30], 64);
    }

    #[test]
    fn vector_backend_subslice_mut_updates_backing_vector() {
        let mut wrapped =
            BitBatchParDataVector::<WordIdx, BoolIdx, u32>::from_vec(vec![1, 2, 3], quant_usize(64));
        let mut sub = wrapped
            .subslice_mut(Range {
                start: quant_word(0),
                end: quant_word(2),
            })
            .expect("vector subslice_mut");
        sub.set_word(quant_word(1), 99).unwrap();
        assert_eq!(wrapped.into_vec(), vec![1, 99, 3]);
    }
}

#[cfg(feature = "heapless")]
mod heapless_backend {
    use super::*;
    use feagi_basis_collections::bit_packed_bool::par_data::BitBatchParDataHeaplessVec;

    #[test]
    fn heapless_backend_new_uniform_and_from_array_store() {
        let uniform =
            BitBatchParDataHeaplessVec::<WordIdx, BoolIdx, u32, 2>::new_uniform(0xABCD, quant_usize(50));
        assert_eq!(uniform.len().quant_to_usize(), 2);
        assert_eq!(uniform.store(), &[0xABCD, 0xABCD]);

        let wrapped = BitBatchParDataHeaplessVec::<WordIdx, BoolIdx, u32, 3>::from_array(
            [1, 2, 3],
            quant_usize(70),
        );
        assert_eq!(wrapped.into_store().into_array(), Ok([1, 2, 3]));
    }

    #[test]
    fn heapless_backend_subslice_matches_word_range_and_valid_bools() {
        let data = BitBatchParDataHeaplessVec::<WordIdx, BoolIdx, u32, 4>::from_array(
            [10, 20, 30, 40],
            quant_usize(100),
        );
        let view = data
            .subslice(Range {
                start: quant_word(1),
                end: quant_word(3),
            })
            .expect("heapless subslice");
        assert_word_subslice(&view, &[20, 30], 64);
    }
}

//endregion
