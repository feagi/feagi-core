use core::marker::PhantomData;
use core::ops::Index;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;

pub struct ParSlice<QI: QuantizedIndexCountTrait, V> {
    slice: *const V,
    phantom_data: PhantomData<QI>
}

unsafe impl<QI: QuantizedIndexCountTrait, V> Send for ParSlice<QI, V> {}
unsafe impl<QI: QuantizedIndexCountTrait, V> Sync for ParSlice<QI, V> {}

impl<QI: QuantizedIndexCountTrait, V> ParSlice<QI, V> {
    pub fn new(slice: &[V]) -> ParSlice<QI, V> {
        Self {
            slice: slice.as_ptr(),
            phantom_data: PhantomData
        }
    }

    unsafe fn get_with_quant(&self, index: QI) -> &V {
        &*self.slice.add(index.to_usize())
    }
}

impl<QI: QuantizedIndexCountTrait, V> From<&Vec<V>> for ParSlice<QI, V> {
    fn from(value: &Vec<V>) -> Self {
        Self::new(value)
    }
}


pub struct ParSliceMut<QI: QuantizedIndexCountTrait, V> {
    slice: *mut V,
    phantom_data: PhantomData<QI>
}

unsafe impl<QI: QuantizedIndexCountTrait, V> Send for ParSliceMut<QI, V> {}
unsafe impl<QI: QuantizedIndexCountTrait, V> Sync for ParSliceMut<QI, V> {}

impl<QI: QuantizedIndexCountTrait, V> ParSliceMut<QI, V> {
    pub fn new(slice: &mut [V]) -> ParSliceMut<QI, V> {
        Self {
            slice: slice.as_mut_ptr(),
            phantom_data: PhantomData
        }
    }

    unsafe fn get_with_quant(&self, index: QI) -> &mut V {
        &mut *self.slice.add(index.to_usize())
    }
}

impl<QI: QuantizedIndexCountTrait, V> From<&mut Vec<V>> for ParSliceMut<QI, V> {
    fn from(mut value: &mut Vec<V>) -> Self {
        Self::new(value)
    }
}