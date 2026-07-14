use core::marker::PhantomData;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialCPUQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron_models::neuron_model_traits::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};

/// Writes neuron data for a cortical area
pub(crate) trait CorticalWriter<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type CorticalModelAndQuant: CorticalPotentialCPUQuantization;
    type CorticalArea: NeuronModelCorticalData<Self::CorticalModelAndQuant>;
    type Neuron: NeuronModelNeuronData<Self::CorticalModelAndQuant>;

    fn write_cortical_and_neuron_data(
        self,
        cortical_write_target: &mut Self::CorticalArea,
        neuron_write_target: &mut [Self::Neuron],
        neuron_filter: impl FnMut(&FIQ::NeuronIndexCountQuant, &mut Self::Neuron) -> bool,
    );
}

pub struct UniformCorticalWriter<FIQ, CPQ, NMCD, NMND>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    NMCD: NeuronModelCorticalData<CPQ>,
    NMND: NeuronModelNeuronData<CPQ>,
{
    cortical_to_write: NMCD,
    neuron_to_write: NMND,
    _p: PhantomData<(FIQ, CPQ)>,
}

impl<FIQ, CPQ, NMCD, NMND> UniformCorticalWriter<FIQ, CPQ, NMCD, NMND>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    NMCD: NeuronModelCorticalData<CPQ>,
    NMND: NeuronModelNeuronData<CPQ>,
{
    pub fn new(cortical_data: NMCD, uniform_neuron: NMND) -> Self {
        Self {
            cortical_to_write: cortical_data,
            neuron_to_write: uniform_neuron,
            _p: PhantomData,
        }
    }
}

impl<FIQ, CPQ, NMCD, NMND> CorticalWriter<FIQ> for UniformCorticalWriter<FIQ, CPQ, NMCD, NMND>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    NMCD: NeuronModelCorticalData<CPQ>,
    NMND: NeuronModelNeuronData<CPQ>,
{
    type CorticalModelAndQuant = CPQ;
    type CorticalArea = NMCD;
    type Neuron = NMND;

    fn write_cortical_and_neuron_data(self, cortical_write_target: &mut Self::CorticalArea, neuron_write_target: &mut [Self::Neuron], neuron_filter: impl FnMut(&FIQ::NeuronIndexCountQuant, &mut Self::Neuron) -> bool) {
        *cortical_write_target = self.cortical_to_write;
        neuron_write_target.iter().filter(neuron_filter).iter_mut().for_each(|neuron| {
            *neuron = self.neuron_to_write.clone();
        })

    }
}
