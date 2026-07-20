use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron::cortical_area_layout::CorticalAreaLayout;
use crate::neuron::models_shared::data::{NeuronModelCorticalData, NeuronModelNeuronData};

/// Describes a struct capable of writing / updating some or all of a cortical area properties. Used
/// for editing cortical areas and also initializing them
pub trait NeuronModelWriter<CPQ: CorticalPotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>> {
    // NOTE: assumes extending off of neuron model specific quantization level

    fn write_cortical_data<FIQ, CAL>(&self, overwriting_cortical_data: &mut NMCD, overwriting_neuron_data: &mut [NMND], layout: &CAL) -> Result<(), ()> where // TODO Result
        FIQ: FeagiIndexQuantization,
        CAL: CorticalAreaLayout<FIQ>
    ;
}

/// A common Uniform Model Writer where we simply write the same neuron for every neuron
/// in a cortical area. Many models will use this as their default neuron writer when
/// creating a new area, and also for when editing properties. If a property is set to null, it
/// is not overwritten
pub struct UniformNeuronModelWriter<CPQ, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>>
where
    CPQ: CorticalPotentialQuantization,
{
    cortical_data: Option<NMCD>,
    uniform_neuron_data: Option<NMND>,
    _p: core::marker::PhantomData<CPQ>
}

impl<CPQ, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>> UniformNeuronModelWriter<CPQ, NMCD, NMND>
where
    CPQ: CorticalPotentialQuantization,
{
    pub fn new(cortical_data: Option<NMCD>, uniform_neuron_data: Option<NMND>) -> Self {
        Self {
            cortical_data,
            uniform_neuron_data,
            _p: core::marker::PhantomData
        }
    }
}

impl<CPQ, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>> NeuronModelWriter<CPQ, NMCD, NMND> for UniformNeuronModelWriter<CPQ, NMCD, NMND>
where
    CPQ: CorticalPotentialQuantization,
{
    fn write_cortical_data<FIQ, CAL>(&self, overwriting_cortical_data: &mut NMCD, overwriting_neuron_data: &mut [NMND], _layout: &CAL) -> Result<(), ()>
    where
        FIQ: FeagiIndexQuantization,
        CAL: CorticalAreaLayout<FIQ>
    {
        if let Some(cortical_data) = &self.cortical_data {
            *overwriting_cortical_data = cortical_data.clone();
        }

        if let Some(uniform_neuron_data) = &self.uniform_neuron_data {
            overwriting_neuron_data.iter_mut().for_each(|neuron| {
                *neuron = uniform_neuron_data.clone();
            })
        }

        Ok(())
    }
}

