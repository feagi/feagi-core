//! Cortical areas fundamental to the running of the brain

use super::{CorticalAreaType, UsesFoundationDeath, UsesFoundationPower};
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::quantization::quantizable::DataValueQuantizationError;
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisDefaultsGenerator, NeuronLayoutOf, NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::foundation_death::{
    FoundationDeathCorticalDataInternal, FoundationDeathCorticalDataProperties, FoundationDeathCorticalDataVariance,
    FoundationDeathNeuronDataInternal, NeuronModelFoundationDeath,
};
use feagi_connectome::connectome_basis::neuron_model::foundation_power::{
    FoundationPowerCorticalDataInternal, FoundationPowerCorticalDataProperties, FoundationPowerCorticalDataVariance,
    FoundationPowerNeuronDataInternal, NeuronModelFoundationPower,
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
    PowerFoundationPowerGenerator for Power, UsesFoundationPower, NeuronModelFoundationPower
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelFoundationPower> PowerFoundationPowerGenerator<FIQ, NM> {
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
            FoundationPowerCorticalDataProperties::new(),
            FoundationPowerCorticalDataInternal::new(),
            FoundationPowerCorticalDataVariance::new(),
            FoundationPowerNeuronDataInternal::new(),
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
    DeathFoundationDeathGenerator for Death, UsesFoundationDeath, NeuronModelFoundationDeath
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelFoundationDeath> DeathFoundationDeathGenerator<FIQ, NM> {
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
            FoundationDeathCorticalDataProperties::new(),
            FoundationDeathCorticalDataInternal::new(),
            FoundationDeathCorticalDataVariance::new(),
            FoundationDeathNeuronDataInternal::new(),
        )))
    }
}

//endregion
