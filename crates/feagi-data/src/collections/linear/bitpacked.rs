use crate::values::quantizable::QuantizedIndexCountTrait;

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