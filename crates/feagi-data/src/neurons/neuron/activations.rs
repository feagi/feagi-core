use crate::collections::linear::contiguous_data::{QuantizedContiguousTrait, QuantizedContiguousVector};
use crate::neurons::neuron::indexing::{NeuronActivationBitBatchCount, NeuronActivationBitBatchIndex};
use crate::values::quantizable::{QuantizedUnsignedIntegerUnwrappedTrait};

/// Encodes if neuron is firing for up to 8 neurons (assuming no padding at the end)
pub struct NeuronBitBatchActivation(u8);

impl NeuronBitBatchActivation {

    // TODO debug check for invalid bit indexes

    pub fn get_bit(&self, bit_index: u8) -> bool {

        (self.0 & (1u8 << bit_index)) != 0
    }

    pub fn set_bit(&mut self, bit_index: u8, value: bool) {
        if value {
            self.0 |= 1 << bit_index;
        } {
            self.0 &= 0 << bit_index;
        }
    }

    pub fn count_ones(&mut self) -> u32 {
        self.0.count_ones()
    }

    pub fn count_zeros(&mut self) -> u32 {
        self.0.count_zeros()
    }
}

// TODO

pub struct NeuronBitBatchActivationLinearVector<QI: QuantizedUnsignedIntegerUnwrappedTrait> {
    data: QuantizedContiguousVector<
        NeuronActivationBitBatchIndex<QI>,
        NeuronActivationBitBatchCount<QI>,
        u8
    >
}


