use crate::neuron::cortical_area_layout::CorticalAreaLayout;
use crate::neuron::shared::data::NeuronModelNeuronData;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::marker::PhantomData;

/// Creates the neuron data for a newly forming cortical area
pub trait CorticalNeuronGenerator<
    FIQ: FeagiIndexQuantization,
    CL: CorticalAreaLayout<FIQ>,
    CPQ: CorticalPotentialQuantization,
    NMND: NeuronModelNeuronData<CPQ>,
>
{
    fn write_new_neuron_data(&self, range: &mut [NMND], layout: &CL);
}

/// Simply writes a copy of the same neuron to all slots
pub struct UniformNeuronGenerator<
    FIQ: FeagiIndexQuantization,
    CL: CorticalAreaLayout<FIQ>,
    CPQ: CorticalPotentialQuantization,
    NMND: NeuronModelNeuronData<CPQ>,
> {
    uniform_neuron: NMND,
    _p: PhantomData<(FIQ, CPQ, CL, NMND)>,
}

impl<FIQ: FeagiIndexQuantization, CL: CorticalAreaLayout<FIQ>, CPQ: CorticalPotentialQuantization, NMND: NeuronModelNeuronData<CPQ>>
    CorticalNeuronGenerator<FIQ, CL, CPQ, NMND> for UniformNeuronGenerator<FIQ, CL, CPQ, NMND>
{
    fn write_new_neuron_data(&self, range: &mut [NMND], layout: &CL) {
        range.iter_mut().for_each(|v| *v = self.uniform_neuron.clone())
    }
}
