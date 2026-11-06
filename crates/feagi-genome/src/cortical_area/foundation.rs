//! Cortical areas fundamental to the running of the brain

use super::{CorticalAreaType, UsesFoundationDeath, UsesFoundationPower};
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::quantization::quantizable::DataValueQuantizationError;
use feagi_connectome::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::foundation_death::{
    FoundationDeathCorticalDataInternal, FoundationDeathCorticalDataProperties, FoundationDeathCorticalDataScratch, FoundationDeathCorticalDataVariance,
    FoundationDeathNeuronDataInternal, FoundationDeathNeuronDataScratch, NeuronModelFoundationDeath,
};
use feagi_connectome::connectome_basis::neuron_model::foundation_power::{
    FoundationPowerCorticalDataInternal, FoundationPowerCorticalDataProperties, FoundationPowerCorticalDataScratch, FoundationPowerCorticalDataVariance,
    FoundationPowerNeuronDataInternal, FoundationPowerNeuronDataScratch, NeuronModelFoundationPower,
};

//region Power

/// Power management processing
pub struct Power;

impl CorticalAreaType for Power {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

impl UsesFoundationPower for Power {
    fn default_universal_properties<NM: NeuronModelFoundationPower>() -> Result<NeuronModelUniversalCorticalProperties<NM>, DataValueQuantizationError> {
        Ok(NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        })
    }

    fn default_cortical_properties<NM: NeuronModelFoundationPower>() -> Result<NM::CorticalDataProperties, DataValueQuantizationError> {
        Ok(FoundationPowerCorticalDataProperties::new())
    }

    fn default_cortical_variance<NM: NeuronModelFoundationPower>() -> Result<NM::CorticalDataVariance, DataValueQuantizationError> {
        Ok(FoundationPowerCorticalDataVariance::new())
    }

    fn cortical_internal<NM: NeuronModelFoundationPower>() -> Result<NM::CorticalDataInternal, DataValueQuantizationError> {
        Ok(FoundationPowerCorticalDataInternal::new())
    }

    fn cortical_scratch<NM: NeuronModelFoundationPower>() -> Result<NM::CorticalDataScratch, DataValueQuantizationError> {
        Ok(FoundationPowerCorticalDataScratch::new())
    }

    fn neuron_internal<NM: NeuronModelFoundationPower>() -> Result<NM::NeuronDataInternal, DataValueQuantizationError> {
        Ok(FoundationPowerNeuronDataInternal::new())
    }

    fn neuron_scratch<NM: NeuronModelFoundationPower>() -> Result<NM::NeuronDataScratch, DataValueQuantizationError> {
        Ok(FoundationPowerNeuronDataScratch::new())
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

impl UsesFoundationDeath for Death {
    fn default_universal_properties<NM: NeuronModelFoundationDeath>() -> Result<NeuronModelUniversalCorticalProperties<NM>, DataValueQuantizationError> {
        Ok(NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        })
    }

    fn default_cortical_properties<NM: NeuronModelFoundationDeath>() -> Result<NM::CorticalDataProperties, DataValueQuantizationError> {
        Ok(FoundationDeathCorticalDataProperties::new())
    }

    fn default_cortical_variance<NM: NeuronModelFoundationDeath>() -> Result<NM::CorticalDataVariance, DataValueQuantizationError> {
        Ok(FoundationDeathCorticalDataVariance::new())
    }

    fn cortical_internal<NM: NeuronModelFoundationDeath>() -> Result<NM::CorticalDataInternal, DataValueQuantizationError> {
        Ok(FoundationDeathCorticalDataInternal::new())
    }

    fn cortical_scratch<NM: NeuronModelFoundationDeath>() -> Result<NM::CorticalDataScratch, DataValueQuantizationError> {
        Ok(FoundationDeathCorticalDataScratch::new())
    }

    fn neuron_internal<NM: NeuronModelFoundationDeath>() -> Result<NM::NeuronDataInternal, DataValueQuantizationError> {
        Ok(FoundationDeathNeuronDataInternal::new())
    }

    fn neuron_scratch<NM: NeuronModelFoundationDeath>() -> Result<NM::NeuronDataScratch, DataValueQuantizationError> {
        Ok(FoundationDeathNeuronDataScratch::new())
    }
}

//endregion
