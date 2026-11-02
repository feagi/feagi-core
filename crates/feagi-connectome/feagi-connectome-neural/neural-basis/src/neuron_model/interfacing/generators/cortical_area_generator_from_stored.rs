use feagi_basis::neurons::CorticalAreaNeuronPotential;
use crate::neuron_model::interfacing::generators::cortical::cortical_area_generator_cortical_from_stored::CorticalAreaGeneratorCorticalFromStored;
use crate::neuron_model::interfacing::generators::neuron::cortical_area_generator_neuron_uniform::CorticalAreaGeneratorNeuronUniform;
use crate::neuron_model::interfacing::generators::CorticalAreaGenerator;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct CorticalAreaGeneratorUniform<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    cortical_area: CorticalAreaGeneratorCorticalFromStored<FIQ, NM>,
    neuron_generator: CorticalAreaGeneratorNeuronUniform<FIQ, NM>,
}

impl<FIQ, NM> CorticalAreaGeneratorUniform<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub fn new(
        cortical_area: CorticalAreaGeneratorCorticalFromStored<FIQ, NM>,
        neuron_generator: CorticalAreaGeneratorNeuronUniform<FIQ, NM>,
    ) -> Self {
        Self {
            cortical_area,
            neuron_generator,
        }
    }
}


impl<FIQ, NM> CorticalAreaGenerator<FIQ, NM> for CorticalAreaGeneratorUniform<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    type CorticalGenerator = CorticalAreaGeneratorCorticalFromStored<FIQ, NM>;
    type NeuronGenerator = CorticalAreaGeneratorNeuronUniform<FIQ, NM>;

    fn get_cortical_generator(&self) -> &Self::CorticalGenerator {
        &self.cortical_area
    }

    fn get_neuron_generator(&self) -> &Self::NeuronGenerator {
        &self.neuron_generator
    }
}
