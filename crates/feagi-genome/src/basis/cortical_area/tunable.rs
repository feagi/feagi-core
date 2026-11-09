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
    self, FeagiAdvancedCorticalDataInternal, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataVariance,
    FeagiAdvancedNeuronDataInternal, NeuronModelFeagiAdvanced,
};
use feagi_connectome::connectome_basis::neuron_model::feagi_simple::{
    self, FeagiSimpleCorticalDataInternal, FeagiSimpleCorticalDataProperties, FeagiSimpleCorticalDataVariance,
    FeagiSimpleNeuronDataInternal, NeuronModelFeagiSimple,
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
    BrightnessAdjustFeagiAdvancedGenerator for BrightnessAdjust, UsesFeagiAdvanced, NeuronModelFeagiAdvanced
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiAdvanced> BrightnessAdjustFeagiAdvancedGenerator<FIQ, NM> {
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
            FeagiAdvancedCorticalDataProperties {
                refractory_period: feagi_advanced::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                leak_coefficient: 1.0f32.try_to_quantization()?,
                leak_variability: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                degeneration: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            FeagiAdvancedCorticalDataInternal::new(),
            FeagiAdvancedCorticalDataVariance {
                firing_threshold_increment_x: 0.0f32.try_to_quantization()?,
                firing_threshold_increment_y: 0.0f32.try_to_quantization()?,
                firing_threshold_increment_z: 0.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            FeagiAdvancedNeuronDataInternal {
                refractory_countdown: feagi_advanced::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

cortical_type_generator! {
    /// The defaults of a new `BrightnessAdjust` area using the `FeagiSimple` neuron model
    BrightnessAdjustFeagiSimpleGenerator for BrightnessAdjust, UsesFeagiSimple, NeuronModelFeagiSimple
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiSimple> BrightnessAdjustFeagiSimpleGenerator<FIQ, NM> {
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
            FeagiSimpleCorticalDataProperties {
                refractory_period: feagi_simple::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            FeagiSimpleCorticalDataInternal::new(),
            FeagiSimpleCorticalDataVariance::new(),
            FeagiSimpleNeuronDataInternal {
                refractory_countdown: feagi_simple::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

//endregion
