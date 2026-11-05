//! Cortical areas fundamental to the running of the brain

use super::{CorticalAreaSpec, CorticalAreaType, FeagiIndexQuantization, UsesFoundationDeath, UsesFoundationPower};
use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::quantization::quantizable::DataValueQuantizationError;
use feagi_connectome::cortical_area::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome::cortical_area::NeuronModelUniversalCorticalProperties;
use feagi_connectome::neuron_models::neuron_model::foundation_death::{
    FoundationDeathCorticalDataInternal, FoundationDeathCorticalDataProperties, FoundationDeathCorticalDataScratch, FoundationDeathCorticalDataVariance,
    FoundationDeathNeuronDataInternal, FoundationDeathNeuronDataScratch, NeuronModelFoundationDeath,
};
use feagi_connectome::neuron_models::neuron_model::foundation_power::{
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
    fn default_spec<FIQ: FeagiIndexQuantization, NM: NeuronModelFoundationPower>() -> Result<CorticalAreaSpec<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaSpec::new_uniform(
            NeuronLayoutVoxel::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize([1, 1, 1, 1]).unwrap()),
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            FoundationPowerCorticalDataProperties::new(),
            FoundationPowerCorticalDataInternal::new(),
            FoundationPowerCorticalDataScratch::new(),
            FoundationPowerCorticalDataVariance::new(),
            FoundationPowerNeuronDataInternal::new(),
            FoundationPowerNeuronDataScratch::new(),
        ))
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
    fn default_spec<FIQ: FeagiIndexQuantization, NM: NeuronModelFoundationDeath>() -> Result<CorticalAreaSpec<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaSpec::new_uniform(
            NeuronLayoutVoxel::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize([1, 1, 1, 1]).unwrap()),
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            FoundationDeathCorticalDataProperties::new(),
            FoundationDeathCorticalDataInternal::new(),
            FoundationDeathCorticalDataScratch::new(),
            FoundationDeathCorticalDataVariance::new(),
            FoundationDeathNeuronDataInternal::new(),
            FoundationDeathNeuronDataScratch::new(),
        ))
    }
}

//endregion
