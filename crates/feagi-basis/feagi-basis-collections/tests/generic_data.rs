use core::ops::{IndexMut, Range};

use feagi_basis_collections::generic_data::par_data::{
    ParData, ParDataArray, ParDataSlice, ParDataSliceMut,
};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;

feagi_basis_quantization::create_wrapped_quantized_unsigned_integer!(pub TestQuant);

type Idx = TestQuant<u16>;

fn idx(value: usize) -> Idx {
    Idx::quant_from_usize_unchecked(value)
}

fn assert_subslice_elements<D: Copy + PartialEq + core::fmt::Debug>(
    view: &ParDataSlice<'_, Idx, D>,
    expected: &[D],
) {
    assert_eq!(view.len(), expected.len());
    assert_eq!(view.iter().copied().collect::<Vec<_>>(), expected);
    for (i, expected_value) in expected.iter().copied().enumerate() {
        assert_eq!(view.get(idx(i)), Some(&expected_value));
    }
}

//region Array backend

#[test]
fn array_backend_new_uniform_and_from_array_match_store() {
    let uniform = ParDataArray::<Idx, u32, 4>::new_uniform(11);
    assert_eq!(uniform.len(), 4);
    assert_eq!(uniform.into_array(), [11, 11, 11, 11]);

    let backing = [1u8, 2, 3];
    let from = ParDataArray::<Idx, u8, 3>::from_array(backing);
    assert_eq!(from.store(), &backing);
    assert_eq!(from.into_array(), backing);
}

#[test]
fn array_backend_subslice_and_subslice_mut_preserve_element_data() {
    let data = ParDataArray::<Idx, i16, 5>::from_array([10, 20, 30, 40, 50]);
    let range = Range {
        start: idx(1),
        end: idx(4),
    };

    let view = data.subslice(range.clone()).expect("in-bounds subslice");
    assert_subslice_elements(&view, &[20, 30, 40]);

    let mut data = ParDataArray::<Idx, i16, 5>::from_array([10, 20, 30, 40, 50]);
    let mut view = data.subslice_mut(range).expect("in-bounds subslice_mut");
    assert_eq!(view.iter().copied().collect::<Vec<_>>(), vec![20, 30, 40]);
    assert_eq!(view.set(idx(1), 999), Some(30));
    assert_eq!(data.into_array(), [10, 20, 999, 40, 50]);
}

//endregion

//region Shared slice backend

#[test]
fn shared_slice_backend_constructors_reference_same_elements() {
    let backing = [5u16, 6, 7];
    let from_new = ParDataSlice::<Idx, u16>::new(&backing);
    assert_subslice_elements(&from_new, &[5, 6, 7]);
    let from_alias = ParDataSlice::from_slice(&backing);
    assert_subslice_elements(&from_alias, &[5, 6, 7]);
    let from_trait: ParDataSlice<'_, Idx, u16> = ParData::from_store(&backing);
    assert_subslice_elements(&from_trait, &[5, 6, 7]);
    let from_impl: ParDataSlice<'_, Idx, u16> = (&backing[..]).into();
    assert_subslice_elements(&from_impl, &[5, 6, 7]);
}

#[test]
fn shared_slice_backend_subslice_from_parent_array_view() {
    let data = ParDataArray::<Idx, u64, 4>::from_array([100, 200, 300, 400]);
    let parent_slice = data.as_data_slice();
    let view = parent_slice
        .subslice(Range {
            start: idx(2),
            end: idx(4),
        })
        .expect("subslice of slice view");
    assert_subslice_elements(&view, &[300, 400]);
}

//endregion

//region Mutable slice backend

#[test]
fn mutable_slice_backend_roundtrip_and_reborrow() {
    let mut backing = [1u64, 2, 3];
    let mut view = ParDataSliceMut::<Idx, u64>::new(&mut backing);
    assert_eq!(view.len(), 3);
    assert_eq!(view.get(idx(2)), Some(&3));

    let mut nested = view.reborrow();
    *nested.index_mut(idx(0)) = 50;
    assert_eq!(backing, [50, 2, 3]);
}

#[test]
fn mutable_slice_backend_subslice_mut_updates_backing_only_in_range() {
    let mut backing = [1i32, 2, 3, 4, 5];
    let mut view = ParDataSliceMut::<Idx, i32>::from_slice_mut(&mut backing);
    let mut sub = view
        .subslice_mut(Range {
            start: idx(1),
            end: idx(4),
        })
        .expect("valid mut subslice");
    assert_eq!(sub.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
    sub.iter_mut().enumerate().for_each(|(i, slot)| *slot = (i as i32 + 1) * 10);
    assert_eq!(backing, [1, 10, 20, 30, 5]);
}

//endregion

#[test]
fn default_builds_empty_array_backing_store() {
    let data = ParDataArray::<Idx, u8, 0>::default();
    assert!(data.is_empty());
}

#[cfg(feature = "alloc")]
mod vector_backend {
    use super::*;
    use feagi_basis_collections::generic_data::par_data::ParDataVector;

    #[test]
    fn vector_backend_new_uniform_from_vec_and_into_vec() {
        let uniform = ParDataVector::<Idx, u32>::new_uniform(idx(3), 7);
        assert_eq!(uniform.len(), 3);
        assert_eq!(uniform.into_vec(), vec![7, 7, 7]);

        let wrapped = ParDataVector::<Idx, u8>::from_vec(vec![1, 2, 3]);
        assert_eq!(wrapped.store(), &vec![1u8, 2, 3][..]);
        assert_eq!(wrapped.into_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn vector_backend_append_and_subslice_data() {
        let mut data = ParDataVector::<Idx, u16>::from_vec(vec![10, 20, 30, 40]);
        data.append(idx(2), 99);
        assert_eq!(data.into_vec(), vec![10, 20, 30, 40, 99, 99]);

        let data = ParDataVector::<Idx, u16>::from_vec(vec![10, 20, 30, 40]);
        let view = data
            .subslice(Range {
                start: idx(1),
                end: idx(3),
            })
            .expect("vector subslice");
        assert_subslice_elements(&view, &[20, 30]);
    }

    #[test]
    fn vector_backend_subslice_mut_and_clone_to_vector() {
        let mut data = ParDataVector::<Idx, i32>::from_vec(vec![1, 2, 3, 4]);
        let mut sub = data
            .subslice_mut(Range {
                start: idx(0),
                end: idx(2),
            })
            .expect("mut subslice");
        sub.set(idx(1), -1).unwrap();
        assert_eq!(data.clone_to_vector().into_vec(), vec![1, -1, 3, 4]);
    }
}

#[cfg(feature = "heapless")]
mod heapless_backend {
    use super::*;
    use feagi_basis_collections::generic_data::par_data::ParDataHeaplessVec;

    #[test]
    fn heapless_backend_new_uniform_from_array_and_into_array() {
        let uniform = ParDataHeaplessVec::<Idx, u8, 3>::new_uniform(4);
        assert_eq!(uniform.len(), 3);
        assert_eq!(uniform.into_array(), [4, 4, 4]);

        let wrapped = ParDataHeaplessVec::<Idx, u16, 2>::from_array([10, 20]);
        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped.into_array(), [10, 20]);
    }

    #[test]
    fn heapless_backend_subslice_matches_element_range() {
        let data = ParDataHeaplessVec::<Idx, i32, 4>::from_array([5, 6, 7, 8]);
        let view = data
            .subslice(Range {
                start: idx(1),
                end: idx(3),
            })
            .expect("heapless subslice");
        assert_subslice_elements(&view, &[6, 7]);
    }
}
