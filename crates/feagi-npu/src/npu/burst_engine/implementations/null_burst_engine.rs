use std::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Does nothing, just for dev purposes rn
pub struct NullBurstEngine<FIQ: FeagiIndexQuantization> {
    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NullBurstEngine<FIQ> {
    pub fn new() -> NullBurstEngine<FIQ> {
        Self{ _p: PhantomData }
    }

    pub fn something(&mut self)
    {
        // nothing for now
    }
}