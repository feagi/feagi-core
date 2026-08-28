use std::marker::PhantomData;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

pub struct NeuronProcessingUnit<NPUIQ: NeuronProcessingUnitIndexQuantization>
{
    _p: PhantomData<NPUIQ>,
}

// TODO port the rest of the stuff

impl<NPUIQ: NeuronProcessingUnitIndexQuantization> NeuronProcessingUnit<NPUIQ> {
    pub fn new() -> NeuronProcessingUnit<NPUIQ> {
        Self {
            _p: PhantomData,
        }
    }
}