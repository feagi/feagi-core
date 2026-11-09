//! Outputs that affect the inputs
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaType, UsesFeagiAdvanced, UsesFeagiSimple};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisDefaultsGenerator, NeuronLayoutOf, NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::{
    self, NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance,
    NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedTrait,
};
use feagi_connectome::connectome_basis::neuron_model::feagi_simple::{
    self, NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalVariance,
    NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleTrait,
};

//region BrightnessAdjust

/// Adjusts the brightness of an input
pub struct BrightnessAdjust;

impl CorticalAreaType for BrightnessAdjust {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `BrightnessAdjust` area using the `FeagiAdvanced` neuron model
    BrightnessAdjustFeagiAdvancedGenerator for BrightnessAdjust, UsesFeagiAdvanced, NMFeagiAdvancedTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiAdvancedTrait> BrightnessAdjustFeagiAdvancedGenerator<FIQ, NM> {
    /// A new `BrightnessAdjust` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisDefaultsGenerator::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFeagiAdvancedDataCorticalProperties {
                refractory_period: feagi_advanced::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                leak_coefficient: 1.0f32.try_to_quantization()?,
                leak_variability: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                degeneration: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            NMFeagiAdvancedDataCorticalInternal::new(),
            NMFeagiAdvancedDataCorticalVariance {
                firing_threshold_increment_x: 0.0f32.try_to_quantization()?,
                firing_threshold_increment_y: 0.0f32.try_to_quantization()?,
                firing_threshold_increment_z: 0.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            NMFeagiAdvancedDataNeuronInternal {
                refractory_countdown: feagi_advanced::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

cortical_type_generator! {
    /// The defaults of a new `BrightnessAdjust` area using the `FeagiSimple` neuron model
    BrightnessAdjustFeagiSimpleGenerator for BrightnessAdjust, UsesFeagiSimple, NMFeagiSimpleTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiSimpleTrait> BrightnessAdjustFeagiSimpleGenerator<FIQ, NM> {
    /// A new `BrightnessAdjust` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisDefaultsGenerator::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFeagiSimpleDataCorticalProperties {
                refractory_period: feagi_simple::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            NMFeagiSimpleDataCorticalInternal::new(),
            NMFeagiSimpleDataCorticalVariance::new(),
            NMFeagiSimpleDataNeuronInternal {
                refractory_countdown: feagi_simple::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

//endregion
