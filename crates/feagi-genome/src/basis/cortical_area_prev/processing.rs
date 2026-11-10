//! Cortical areas that process activity within the brain
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaType, UsesFeagiAdvanced, UsesFeagiSimple, UsesMemoryFireHashing};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisGeneratorUniformDefault, NeuronLayoutOf, NeuronLayoutTagSingleSteppedDynamic, NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
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
use feagi_connectome::neuron_models::models::::{
    self, NMMemoryFireHashingDataCorticalInternal, NMMemoryFireHashingDataCorticalProperties, NMMemoryFireHashingDataCorticalVariance,
    NMMemoryFireHashingDataNeuronInternal, NMMemoryFireHashingTrait,
};

//region Interconnect

/// Connects cortical areas to each other
pub struct Interconnect;

impl CorticalAreaType for Interconnect {
    type NeuronLayout = NeuronLayoutTagVoxel;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `Interconnect` area using the `FeagiAdvanced` neuron model
    InterconnectFeagiAdvancedGenerator for Interconnect, UsesFeagiAdvanced, NMFeagiAdvancedTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiAdvancedTrait> InterconnectFeagiAdvancedGenerator<FIQ, NM> {
    /// A new `Interconnect` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
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
    /// The defaults of a new `Interconnect` area using the `FeagiSimple` neuron model
    InterconnectFeagiSimpleGenerator for Interconnect, UsesFeagiSimple, NMFeagiSimpleTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMFeagiSimpleTrait> InterconnectFeagiSimpleGenerator<FIQ, NM> {
    /// A new `Interconnect` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
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

//region Memory

/// Stores information and grows dynamically. New areas start without any neurons
pub struct Memory;

impl CorticalAreaType for Memory {
    type NeuronLayout = NeuronLayoutTagSingleSteppedDynamic;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `Memory` area using the `MemoryFireHashing` neuron model
    MemoryMemoryFireHashingGenerator for Memory, UsesMemoryFireHashing, NMMemoryFireHashingTrait
}

impl<FIQ: FeagiIndexQuantization, NM: NMMemoryFireHashingTrait> MemoryMemoryFireHashingGenerator<FIQ, NM> {
    /// A new `Memory` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisGeneratorUniformDefault::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMMemoryFireHashingDataCorticalProperties {
                refractory_period: ::Period::QUANT_ONE,
                firing_threshold: 1.0f32.try_to_quantization()?,
                postsynaptic_current: 1.0f32.try_to_quantization()?,
                postsynaptic_current_max: 1.0f32.try_to_quantization()?,
                _p: PhantomData,
            },
            NMMemoryFireHashingDataCorticalInternal::new(),
            NMMemoryFireHashingDataCorticalVariance::new(),
            NMMemoryFireHashingDataNeuronInternal {
                refractory_countdown: ::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

//endregion
