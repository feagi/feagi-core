//! Cortical areas that process activity within the brain
//!
//! TODO the default values here are placeholders, chosen only to be valid at every quantization

use super::{CorticalAreaType, UsesFeagiAdvanced, UsesFeagiSimple, UsesMemoryFireHashing};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::quantization::quantizable::{DataValueQuantizationError, QuantizedDecimalTrait};
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisDefaultsGenerator, NeuronLayoutOf, NeuronLayoutTagSingleSteppedDynamic, NeuronLayoutTagVoxel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::{
    self, FeagiAdvancedCorticalDataInternal, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataVariance,
    FeagiAdvancedNeuronDataInternal, NeuronModelFeagiAdvanced,
};
use feagi_connectome::connectome_basis::neuron_model::feagi_simple::{
    self, FeagiSimpleCorticalDataInternal, FeagiSimpleCorticalDataProperties, FeagiSimpleCorticalDataVariance,
    FeagiSimpleNeuronDataInternal, NeuronModelFeagiSimple,
};
use feagi_connectome::connectome_basis::neuron_model::memory_fire_hashing::{
    self, MemoryFireHashingCorticalDataInternal, MemoryFireHashingCorticalDataProperties, MemoryFireHashingCorticalDataVariance,
    MemoryFireHashingNeuronDataInternal, NeuronModelMemoryFireHashing,
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
    InterconnectFeagiAdvancedGenerator for Interconnect, UsesFeagiAdvanced, NeuronModelFeagiAdvanced
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiAdvanced> InterconnectFeagiAdvancedGenerator<FIQ, NM> {
    /// A new `Interconnect` area laid out as `layout`, starting from the defaults of this type. Errors if
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
    /// The defaults of a new `Interconnect` area using the `FeagiSimple` neuron model
    InterconnectFeagiSimpleGenerator for Interconnect, UsesFeagiSimple, NeuronModelFeagiSimple
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelFeagiSimple> InterconnectFeagiSimpleGenerator<FIQ, NM> {
    /// A new `Interconnect` area laid out as `layout`, starting from the defaults of this type. Errors if
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

//region Memory

/// Stores information and grows dynamically. New areas start without any neurons
pub struct Memory;

impl CorticalAreaType for Memory {
    type NeuronLayout = NeuronLayoutTagSingleSteppedDynamic;
    type Inner = [u8; 6];
}

cortical_type_generator! {
    /// The defaults of a new `Memory` area using the `MemoryFireHashing` neuron model
    MemoryMemoryFireHashingGenerator for Memory, UsesMemoryFireHashing, NeuronModelMemoryFireHashing
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelMemoryFireHashing> MemoryMemoryFireHashingGenerator<FIQ, NM> {
    /// A new `Memory` area laid out as `layout`, starting from the defaults of this type. Errors if
    /// a default decimal becomes infinite or NaN at quantization `NM`
    pub fn new(layout: NeuronLayoutOf<NM, FIQ>) -> Result<Self, DataValueQuantizationError> {
        Ok(Self(NeurogenesisDefaultsGenerator::new(
            layout,
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
            MemoryFireHashingCorticalDataVariance::new(),
            MemoryFireHashingNeuronDataInternal {
                refractory_countdown: memory_fire_hashing::Period::QUANT_ZERO,
                _p: PhantomData,
            },
        )))
    }
}

//endregion
