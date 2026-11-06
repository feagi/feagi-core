//! Outputs that affect the inputs
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaType, UsesFeagiAdvanced, UsesFeagiSimple};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::{
    self, FeagiAdvancedCorticalDataInternal, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataScratch, FeagiAdvancedCorticalDataVariance,
    FeagiAdvancedNeuronDataInternal, FeagiAdvancedNeuronDataScratch, NeuronModelFeagiAdvanced,
};
use feagi_connectome::connectome_basis::neuron_model::feagi_simple::{
    self, FeagiSimpleCorticalDataInternal, FeagiSimpleCorticalDataProperties, FeagiSimpleCorticalDataScratch, FeagiSimpleCorticalDataVariance,
    FeagiSimpleNeuronDataInternal, FeagiSimpleNeuronDataScratch, NeuronModelFeagiSimple,
};

//region BrightnessAdjust

/// Adjusts the brightness of an input
pub struct BrightnessAdjust;

impl CorticalAreaType for BrightnessAdjust {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

impl UsesFeagiAdvanced for BrightnessAdjust {
    fn default_universal_properties<NM: NeuronModelFeagiAdvanced>() -> Result<NeuronModelUniversalCorticalProperties<NM>, DataValueQuantizationError> {
        Ok(NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        })
    }

    fn default_cortical_properties<NM: NeuronModelFeagiAdvanced>() -> Result<NM::CorticalDataProperties, DataValueQuantizationError> {
        Ok(FeagiAdvancedCorticalDataProperties {
            refractory_period: feagi_advanced::Period::QUANT_ONE,
            firing_threshold: 1.0f32.try_to_quantization()?,
            leak_coefficient: 1.0f32.try_to_quantization()?,
            leak_variability: 1.0f32.try_to_quantization()?,
            postsynaptic_current: 1.0f32.try_to_quantization()?,
            postsynaptic_current_max: 1.0f32.try_to_quantization()?,
            degeneration: 1.0f32.try_to_quantization()?,
            _p: PhantomData,
        })
    }

    fn default_cortical_variance<NM: NeuronModelFeagiAdvanced>() -> Result<NM::CorticalDataVariance, DataValueQuantizationError> {
        Ok(FeagiAdvancedCorticalDataVariance {
            firing_threshold_increment_x: 0.0f32.try_to_quantization()?,
            firing_threshold_increment_y: 0.0f32.try_to_quantization()?,
            firing_threshold_increment_z: 0.0f32.try_to_quantization()?,
            _p: PhantomData,
        })
    }

    fn cortical_internal<NM: NeuronModelFeagiAdvanced>() -> Result<NM::CorticalDataInternal, DataValueQuantizationError> {
        Ok(FeagiAdvancedCorticalDataInternal::new())
    }

    fn cortical_scratch<NM: NeuronModelFeagiAdvanced>() -> Result<NM::CorticalDataScratch, DataValueQuantizationError> {
        Ok(FeagiAdvancedCorticalDataScratch::new())
    }

    fn neuron_internal<NM: NeuronModelFeagiAdvanced>() -> Result<NM::NeuronDataInternal, DataValueQuantizationError> {
        Ok(FeagiAdvancedNeuronDataInternal {
            refractory_countdown: feagi_advanced::Period::QUANT_ZERO,
            _p: PhantomData,
        })
    }

    fn neuron_scratch<NM: NeuronModelFeagiAdvanced>() -> Result<NM::NeuronDataScratch, DataValueQuantizationError> {
        Ok(FeagiAdvancedNeuronDataScratch::new())
    }
}

impl UsesFeagiSimple for BrightnessAdjust {
    fn default_universal_properties<NM: NeuronModelFeagiSimple>() -> Result<NeuronModelUniversalCorticalProperties<NM>, DataValueQuantizationError> {
        Ok(NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        })
    }

    fn default_cortical_properties<NM: NeuronModelFeagiSimple>() -> Result<NM::CorticalDataProperties, DataValueQuantizationError> {
        Ok(FeagiSimpleCorticalDataProperties {
            refractory_period: feagi_simple::Period::QUANT_ONE,
            firing_threshold: 1.0f32.try_to_quantization()?,
            postsynaptic_current: 1.0f32.try_to_quantization()?,
            postsynaptic_current_max: 1.0f32.try_to_quantization()?,
            _p: PhantomData,
        })
    }

    fn default_cortical_variance<NM: NeuronModelFeagiSimple>() -> Result<NM::CorticalDataVariance, DataValueQuantizationError> {
        Ok(FeagiSimpleCorticalDataVariance::new())
    }

    fn cortical_internal<NM: NeuronModelFeagiSimple>() -> Result<NM::CorticalDataInternal, DataValueQuantizationError> {
        Ok(FeagiSimpleCorticalDataInternal::new())
    }

    fn cortical_scratch<NM: NeuronModelFeagiSimple>() -> Result<NM::CorticalDataScratch, DataValueQuantizationError> {
        Ok(FeagiSimpleCorticalDataScratch::new())
    }

    fn neuron_internal<NM: NeuronModelFeagiSimple>() -> Result<NM::NeuronDataInternal, DataValueQuantizationError> {
        Ok(FeagiSimpleNeuronDataInternal {
            refractory_countdown: feagi_simple::Period::QUANT_ZERO,
            _p: PhantomData,
        })
    }

    fn neuron_scratch<NM: NeuronModelFeagiSimple>() -> Result<NM::NeuronDataScratch, DataValueQuantizationError> {
        Ok(FeagiSimpleNeuronDataScratch::new())
    }
}

//endregion
