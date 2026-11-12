//! Cortical areas receiving input from outside of the brain
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaType, UsesFeagiAdvanced, UsesFeagiSimple};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisGeneratorUniformDefault, NeuronLayoutOf, NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neuron_models::models::;
use feagi_connectome::neuron_models::models::::{
    self, NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance,
    NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedTrait,
};
use feagi_connectome::neuron_models::models::::{
    self, NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalVariance,
    NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleTrait,
};

//region Camera

/// Image input
pub struct Camera;

impl CorticalAreaType for Camera {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `Camera` area using the `FeagiAdvanced` neuron model
    CameraFeagiAdvancedGenerator for Camera, UsesFeagiAdvanced, NMFeagiAdvancedTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiAdvancedTrait> CameraFeagiAdvancedGenerator<FIQ, NM> {
    /// A new `Camera` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<FIQ, NM>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisGeneratorUniformDefault::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFeagiAdvancedDataCorticalProperties {
                refractory_period: ::Period::QUANT_ONE,
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
                refractory_countdown: ::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

cortical_type_generator! {
    /// The defaults of a new `Camera` area using the `FeagiSimple` neuron model
    CameraFeagiSimpleGenerator for Camera, UsesFeagiSimple, NMFeagiSimpleTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiSimpleTrait> CameraFeagiSimpleGenerator<FIQ, NM> {
    /// A new `Camera` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<FIQ, NM>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisGeneratorUniformDefault::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFeagiSimpleDataCorticalProperties {
                refractory_period: ::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            NMFeagiSimpleDataCorticalInternal::new(),
            NMFeagiSimpleDataCorticalVariance::new(),
            NMFeagiSimpleDataNeuronInternal {
                refractory_countdown: ::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

//endregion
