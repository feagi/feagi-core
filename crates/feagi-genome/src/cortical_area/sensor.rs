//! Cortical areas receiving input from outside of the brain
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaInit, CorticalAreaType, FeagiIndexQuantization, UsesFeagiAdvanced, UsesFeagiSimple};
use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::cortical_area::neuron_layout::voxel::{NeuronLayoutTagVoxel, NeuronLayoutVoxel};
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome::cortical_area::NeuronModelUniversalCorticalProperties;
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::{
    self, FeagiAdvancedCorticalDataInternal, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataScratch, FeagiAdvancedCorticalDataVariance,
    FeagiAdvancedNeuronDataInternal, FeagiAdvancedNeuronDataScratch, NeuronModelFeagiAdvanced,
};
use feagi_connectome::neuron_models::neuron_model::feagi_simple::{
    self, FeagiSimpleCorticalDataInternal, FeagiSimpleCorticalDataProperties, FeagiSimpleCorticalDataScratch, FeagiSimpleCorticalDataVariance,
    FeagiSimpleNeuronDataInternal, FeagiSimpleNeuronDataScratch, NeuronModelFeagiSimple,
};

//region Camera

/// Image input
pub struct Camera;

impl CorticalAreaType for Camera {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

impl UsesFeagiAdvanced for Camera {
    fn default_init<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiAdvanced>() -> Result<CorticalAreaInit<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaInit::new_uniform(
            NeuronLayoutVoxel::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize([1, 1, 1, 1]).unwrap()),
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
            FeagiAdvancedCorticalDataScratch::new(),
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
            FeagiAdvancedNeuronDataScratch::new(),
        ))
    }
}

impl UsesFeagiSimple for Camera {
    fn default_init<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiSimple>() -> Result<CorticalAreaInit<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaInit::new_uniform(
            NeuronLayoutVoxel::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize([1, 1, 1, 1]).unwrap()),
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
            FeagiSimpleCorticalDataScratch::new(),
            FeagiSimpleCorticalDataVariance::new(),
            FeagiSimpleNeuronDataInternal {
                refractory_countdown: feagi_simple::Period::QUANT_ZERO,
                _p: PhantomData,
            },
            FeagiSimpleNeuronDataScratch::new(),
        ))
    }
}

//endregion
