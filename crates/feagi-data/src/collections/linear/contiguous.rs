use core::marker::PhantomData;
use crate::values::quantizable::QuantizedIndexCountTrait;

// TODO dense trait
// TODO new with custom generator (iter)
// TODO with iter
// TODO getters, setters, also with slices, with checks, iterators


pub struct QuantizedContiguousVector<QI: QuantizedIndexCountTrait, V: Clone> {
    pub(crate) data: Vec<V>,
    phantom_data: PhantomData<QI>
}

impl<QI: QuantizedIndexCountTrait, V: Clone> QuantizedContiguousVector<QI, V>
{
    pub fn new_uniform(number_values: QI, filling_value: V) -> QuantizedContiguousVector<QI, V> {
        // TODO ensure length isnt 0!

        let values = vec![filling_value; number_values.to_usize()];
        Self {
            data: values,
            phantom_data: PhantomData
        }
    }
    
}

pub struct BitPackedVector<QI: QuantizedIndexCountTrait> {
    pub(crate) data: Vec<u8>, // length is number of bytes
    pub(crate) number_bits: QI,
}

impl<QI: QuantizedIndexCountTrait> BitPackedVector<QI> {

    pub fn new_uniform(number_booleans: QI, initial_state: bool) -> BitPackedVector<QI> {
        let length = QuantizedIndexCountTrait::number_bits_to_number_bytes(number_booleans);
        let values: Vec<u8> = if initial_state
        { vec![255; length.to_usize()] }
        else { vec![0; length.to_usize()] };

        Self {
            data: values,
            number_bits: length
        }
    }
}