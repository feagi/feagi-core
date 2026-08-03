use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;
use crate::wrapped_indexes::BurstIndex;



/// Allows getting historical context to a single neuron. Note that the implied structs are
/// transient and are not stored!
pub trait NeuronFireHistory<FIQ: FeagiIndexQuantization> {
    const TYPE: NeuronFireHistoryType;

    /// Insert a new fire (or not) into the fire history
    unsafe fn push_new_firing(&mut self, is_firing: bool);


    fn get_number_times_fired_in_window(&self) -> usize;
}

/// Denotes what type of fire history is being kept for neurons
#[repr(u8)]
pub enum NeuronFireHistoryType {
    None = 0,
    FireCount = 1,
    FireSequence = 2
}

/// Contains no fire history at all

pub struct NeuronFireHistoryNone();

impl NeuronFireHistoryNone {
    pub fn new() -> NeuronFireHistoryNone {
        NeuronFireHistoryNone()
    }
}

impl<FIQ: FeagiIndexQuantization> NeuronFireHistory<FIQ> for NeuronFireHistoryNone {
    const TYPE: NeuronFireHistoryType = NeuronFireHistoryType::None;

    unsafe fn push_new_firing(&mut self, _is_firing: bool) {
        panic!("None Neuron History cannot have a fire history pushed to it!")
    }

    fn get_number_times_fired_in_window(&self) -> usize {
        0
    }
}

///
pub struct NeuronFireHistoryFireCount<'a, FIQ: FeagiIndexQuantization> {
    firings_slice: &'a mut [u8],
    current_byte_index: usize,
    current_bit_index: u8,

    _p: PhantomData<FIQ>,
}

impl<'a, FIQ: FeagiIndexQuantization> NeuronFireHistoryFireCount<'a, FIQ> {
    pub fn new(firings_slice: &'a mut [u8], current_byte_index: usize, current_bit_index: u8) -> Self {
        Self { firings_slice, current_byte_index, current_bit_index, _p: PhantomData }
    }
}

impl<'a, FIQ: FeagiIndexQuantization> NeuronFireHistoryFireCount<'a, FIQ> {

}

impl<FIQ: FeagiIndexQuantization> NeuronFireHistory<FIQ> for NeuronFireHistoryFireCount<'_, FIQ> {
    const TYPE: NeuronFireHistoryType = NeuronFireHistoryType::FireCount;

    unsafe fn push_new_firing(&mut self, is_firing: bool) {
        let mask: u8 = 1 << self.current_bit_index;
        let byte = self.firings_slice.get_unchecked_mut(self.current_byte_index);
        if is_firing {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }

    }

    fn get_number_times_fired_in_window(&self) -> usize {
        self.firings_slice.iter().map(|b| b.count_ones() as usize).sum()
    }
}
