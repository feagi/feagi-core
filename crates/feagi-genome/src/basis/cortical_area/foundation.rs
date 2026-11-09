//! Cortical areas fundamental to the running of the brain

use super::{CorticalAreaType, UsesFoundationDeath, UsesFoundationPower};
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::quantization::quantizable::DataValueQuantizationError;
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisDefaultsGenerator, NeuronLayoutOf, NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::foundation_death::{
    NMFoundationDeathDataCorticalInternal, NMFoundationDeathDataCorticalProperties, NMFoundationDeathDataCorticalVariance,
    NMFoundationDeathDataNeuronInternal, NMFoundationDeathTrait,
};
use feagi_connectome::connectome_basis::neuron_model::foundation_power::{
    NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalVariance,
    NMFoundationPowerDataNeuronInternal, NMFoundationPowerTrait,
};

//region Power

/// Power management processing
pub struct Power;

impl CorticalAreaType for Power {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `Power` area using the `FoundationPower` neuron model
    PowerFoundationPowerGenerator for Power, UsesFoundationPower, NMFoundationPowerTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFoundationPowerTrait> PowerFoundationPowerGenerator<FIQ, NM> {
    /// A new `Power` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisDefaultsGenerator::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFoundationPowerDataCorticalProperties::new(),
            NMFoundationPowerDataCorticalInternal::new(),
            NMFoundationPowerDataCorticalVariance::new(),
            NMFoundationPowerDataNeuronInternal::new(),
        )))
    }
}

//endregion

//region Death

/// Termination of the brain
pub struct Death;

impl CorticalAreaType for Death {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `Death` area using the `FoundationDeath` neuron model
    DeathFoundationDeathGenerator for Death, UsesFoundationDeath, NMFoundationDeathTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFoundationDeathTrait> DeathFoundationDeathGenerator<FIQ, NM> {
    /// A new `Death` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisDefaultsGenerator::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFoundationDeathDataCorticalProperties::new(),
            NMFoundationDeathDataCorticalInternal::new(),
            NMFoundationDeathDataCorticalVariance::new(),
            NMFoundationDeathDataNeuronInternal::new(),
        )))
    }
}

//endregion
