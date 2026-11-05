//! Cortical areas that process activity within the brain
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaSpec, CorticalAreaType, FeagiIndexQuantization, UsesFeagiAdvanced, UsesFeagiSimple, UsesMemoryFireHashing};
use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::cortical_area::neuron_layout::single_stepped_dynamic::{NeuronLayoutSingleSteppedDynamic, NeuronLayoutTagSingleSteppedDynamic};
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
use feagi_connectome::neuron_models::neuron_model::memory_fire_hashing::{
    self, MemoryFireHashingCorticalDataInternal, MemoryFireHashingCorticalDataProperties, MemoryFireHashingCorticalDataScratch, MemoryFireHashingCorticalDataVariance,
    MemoryFireHashingNeuronDataInternal, MemoryFireHashingNeuronDataScratch, NeuronModelMemoryFireHashing,
};

//region Interconnect

/// Connects cortical areas to each other
pub struct Interconnect;

impl CorticalAreaType for Interconnect {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

impl UsesFeagiAdvanced for Interconnect {
    fn default_spec<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiAdvanced>() -> Result<CorticalAreaSpec<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaSpec::new_uniform(
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

impl UsesFeagiSimple for Interconnect {
    fn default_spec<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiSimple>() -> Result<CorticalAreaSpec<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaSpec::new_uniform(
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

//region Memory

/// Stores information and grows dynamically
pub struct Memory;

impl CorticalAreaType for Memory {
    type NeuronLayout = NeuronLayoutTagSingleSteppedDynamic;
    type Inner = [u8; 6];
}

impl UsesMemoryFireHashing for Memory {
    /// Starts without any neurons, as memory grows dynamically
    fn default_spec<FIQ: FeagiIndexQuantization, NM: NeuronModelMemoryFireHashing>() -> Result<CorticalAreaSpec<FIQ, NM>, DataValueQuantizationError> {
        Ok(CorticalAreaSpec::new_uniform(
            NeuronLayoutSingleSteppedDynamic::new(CorticalAreaNeuronCount::QUANT_ZERO),
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            MemoryFireHashingCorticalDataProperties {
                refractory_period: memory_fire_hashing::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            MemoryFireHashingCorticalDataInternal::new(),
            MemoryFireHashingCorticalDataScratch::new(),
            MemoryFireHashingCorticalDataVariance::new(),
            MemoryFireHashingNeuronDataInternal {
                refractory_countdown: memory_fire_hashing::Period::QUANT_ZERO,
                _p: PhantomData,
            },
            MemoryFireHashingNeuronDataScratch::new(),
        ))
    }
}

//endregion
