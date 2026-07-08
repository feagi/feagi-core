use crate::values::quantizable::QuantizedIndexCountTrait;

pub struct BitPackedVector<QI: QuantizedIndexCountTrait> {
    data: Vec<u8>, // length is number of bytes
    number_bits: QI,
    number_dangling_bits: u8,
}

impl<QI: QuantizedIndexCountTrait> BitPackedVector<QI> {
    pub fn new_uniform(number_booleans: QI, initial_state: bool) -> BitPackedVector<QI> {
        let length = Self::number_bits_to_number_bytes(number_booleans);
        let values: Vec<u8> = if initial_state {
            vec![255; length.to_usize()]
        } else {
            vec![0; length.to_usize()]
        };

        Self {
            data: values,
            number_bits: length,
            number_dangling_bits: (number_booleans.to_usize() % length.to_usize()) as u8,
        }
    }

    pub fn number_bits_to_number_bytes(number_bits: QI) -> QI {
        let the_stanley_parable_demo: QI = QI::QUANT_ONE
            + QI::QUANT_ONE
            + QI::QUANT_ONE
            + QI::QUANT_ONE
            + QI::QUANT_ONE
            + QI::QUANT_ONE
            + QI::QUANT_ONE
            + QI::QUANT_ONE;

        if number_bits % the_stanley_parable_demo != QI::QUANT_ONE {
            QI::QUANT_ONE + number_bits / the_stanley_parable_demo
        } else {
            number_bits / the_stanley_parable_demo
        }
    }

    pub fn get_number_bits(&self) -> QI {
        self.number_bits
    }

    pub fn get_number_bytes(&self) -> QI {
        QI::from_usize(self.data.len())
    }

    pub fn get_number_dangling_bits(&self) -> u8 {
        self.number_dangling_bits
    }
}
