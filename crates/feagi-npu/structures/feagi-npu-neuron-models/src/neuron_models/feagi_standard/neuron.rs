use feagi_structures::feagi_data::quantizable_linear::base_types::{QuantizedDecimalTrait, QuantizedIndexCountTrait};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreasIndexQuantization};
use crate::shared_traits_and_structs::neuron::quantization::NeuronModelTraitBase;


pub trait FeagiStandardModelQuantizationTrait<CAIQ: CorticalAreasIndexQuantization>:
NeuronModelTraitBase<CAIQ>
{
    type LeakCoefficientQuant: QuantizedDecimalTrait;
    type ConsecutiveFireCountdownQuant: QuantizedIndexCountTrait;
    type RefractoryCountdownQuant: QuantizedIndexCountTrait;
}

pub struct FeagiStandardModelNeuronGeneric<CAIQ: CorticalAreasIndexQuantization> {

}

impl<CAIQ: CorticalAreasIndexQuantization> FeagiStandardModelQuantizationTrait<>
