use feagi_basis::neurons::CorticalAreaNeuronPotential;
use crate::neuron_model::interfacing::generators::cortical::cortical_area_generator_cortical_from_stored::CorticalAreaGeneratorCorticalFromStored;
use crate::neuron_model::interfacing::generators::neuron::cortical_area_generator_neuron_uniform::CorticalAreaGeneratorNeuronUniform;
use crate::neuron_model::interfacing::generators::CorticalAreaGenerator;
use crate::neuron_model::interfacing::shared::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct CorticalAreaGeneratorFromDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    cortical_area: CorticalAreaGeneratorCorticalFromStored<FIQ, NM>,
    neuron_generator: CorticalAreaGeneratorNeuronUniform<FIQ, NM>,
}

impl<FIQ, NM> CorticalAreaGeneratorFromDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    
    pub fn new_from_defaults_spawn_data(
        default_spawn_data: CorticalModelDefaultsSpawnData<FIQ, NM>
    ) -> Self {
        let cortical_stored = 
            CorticalAreaGeneratorCorticalFromStored::new_manual(
                default_spawn_data.layout.clone(),
                default_spawn_data.universal_data,
                default_spawn_data.cortical_properties,
                default_spawn_data.cortical_internal,
                default_spawn_data.cortical_scratch
            );
        let neuron_uniform = CorticalAreaGeneratorNeuronUniform::new(
            default_spawn_data.layout,
            default_spawn_data.neuron_internal,
            default_spawn_data.neuron_scratch
        );
        Self {
            cortical_area: cortical_stored,
            neuron_generator: neuron_uniform,
        }
    }
}

impl<FIQ, NM> CorticalAreaGenerator<FIQ, NM> for CorticalAreaGeneratorFromDefaults<FIQ, NM>
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




/// Every piece of data needed to spawn a new cortical area, at the given model quantization via 
/// the defaults spawner. Built from a set of model defaults and the given cortical properties
#[derive(Debug, Clone, PartialEq)] // No need to serialize this, this is transient
pub struct CorticalModelDefaultsSpawnData<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: NeuronLayoutOf<NM, FIQ>,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: NM::CorticalDataInternal,
    /// Runtime cortical level data, not saved
    pub cortical_scratch: NM::CorticalDataScratch,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: NM::CorticalFunctionContext,
    /// The data every neuron starts with that is saved
    pub neuron_internal: NM::NeuronDataInternal,
    /// The data every neuron starts with that is not saved
    pub neuron_scratch: NM::NeuronDataScratch,
    pub universal_data: NeuronModelUniversalCorticalProperties<NM>
}