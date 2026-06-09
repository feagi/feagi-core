use crate::quantizable_linear::base_types::QuantizedDecimalTrait;
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::SupportsUintOps;


#[repr(u8)]
#[derive(Default)]
pub enum FeagiGlobalQuantizationLevel {
    #[default]
    Standard = 0,
    // TODO tiny, mini, big, absurd

}

#[repr(u8)]
#[derive(Default)]
pub enum CorticalPotentialQuantizationLevel {
    #[default]
    Float32 = 0,
    // TODO f16, f8, uint8, sint8, f64
}

#[repr(u8)]
#[derive(Default)]
pub enum FeagiGlobalCorticalPotentialQuantizationLevelFlat {
    #[default]
    StandardFloat32 = 0
}

impl FeagiGlobalCorticalPotentialQuantizationLevelFlat {
    pub fn split(&self) -> (FeagiGlobalQuantizationLevel, CorticalPotentialQuantizationLevel)
    {
        match self {
            FeagiGlobalCorticalPotentialQuantizationLevelFlat::StandardFloat32 =>
                {
                    (FeagiGlobalQuantizationLevel::Standard, CorticalPotentialQuantizationLevel::Float32)
                }
        }
    }
}



//region Feagi Global Quantization

/// Global Indexing across an instance of FEAGI, primarily NPU. Controlled by NPU primarily
pub trait FeagiGlobalQuantization {

    const QUANTIZATION_LEVEL: FeagiGlobalQuantizationLevel;

    /// Defines the quantization of the NPU global burst index. This is not model configurable,
    /// rather its in sync with the global setting but also put here since some neuron models need
    /// to have this information to store "burst of last X" as a property
    type GlobalBurstIndexQuant: QuantizedIndexCountTrait;

    /// Neuron linear indexing, linear count, voxel indexing,
    /// and voxel count quantization
    type NeuronIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of synapses within the NPU
    type SynapseIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of cortical areas within the NPU. Note that indexes are not stable outside the NPU!
    type CorticalAreaIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing of axon bundles within the NPU. Note that indexes are not stable outside the NPU!
    type AxonBundleIndexCountQuant: QuantizedIndexCountTrait;

    /// Indexing along the FCLC (the primary or its extensions)
    type FireCandidateListCacheIndexCountQuant: QuantizedIndexCountTrait;

    fn max_number_neurons_globally() -> usize {
        Self::NeuronIndexCountQuant::QUANT_MAX_AS_USIZE
    }

    // TODO other max readings
    // TODO per type indexing?
}


//region Discrete Levels

/// The default quantization level for most deployments. Practical balance between speed and
/// indexing size. The only level supported by some platforms
pub struct FeagiGlobalQuantizationStandard;

impl FeagiGlobalQuantization for FeagiGlobalQuantizationStandard {
    const QUANTIZATION_LEVEL: FeagiGlobalQuantizationLevel = FeagiGlobalQuantizationLevel::Standard;
    type GlobalBurstIndexQuant = u32;
    type NeuronIndexCountQuant = u32;
    type SynapseIndexCountQuant = u32;
    type CorticalAreaIndexCountQuant = u16;
    type AxonBundleIndexCountQuant = u16;
    type FireCandidateListCacheIndexCountQuant = u32;
}

/*
pub struct FeagiGlobalQuantizationMini;
impl FeagiGlobalQuantization for FeagiGlobalQuantizationMini {
    type GlobalBurstIndexQuant = u32;
    type NeuronIndexCountQuant = u16;
    type SynapseIndexCountQuant = u16;
    type CorticalAreaIndexCountQuant = u16;
    type AxonBundleIndexCountQuant = u8;
}


 */


//endregion

//endregion


//region Cortical Potential Quantization
/// Defines the quantization of the neuron potential for a specific cortical area. All cortical
/// areas must have the neuron potential. This quantization is set per cortical area, and
/// is controlled by the Neuron Model quantization state, although this cortical level neuron
/// potential quantization has discrete steps that must be followed.

pub trait CorticalPotentialQuantization {
    const QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel;

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model. This also impacts
    /// the FCL as well
    type NeuronPotentialQuant: QuantizedDecimalTrait;
}

//region Discrete Levels

pub struct CorticalPotentialQuantizationFloat32;

impl CorticalPotentialQuantization for CorticalPotentialQuantizationFloat32 {
    const QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel = CorticalPotentialQuantizationLevel::Float32;
    type NeuronPotentialQuant = f32;
}

//endregion


//endregion


//region Neuron Model Quantization

/// Defines the quantization used in a cortical area for the calculation of neuron dynamics.
/// All are required to support neuron potentials, hence this is the shared base of each model's
/// implementation. Each cortical area within an NPU may have different quantization levels.
/// DO NOT IMPLEMENT THIS IN ACTUAL DATA STRUCTURES! THIS IS ONLY INTENDED TO CARRY QUANTIZATION
/// CONTEXTS
pub trait NeuronModelQuantization
{
    const CORTICAL_POTENTIAL_QUANTIZATION_LEVEL: CorticalPotentialQuantizationLevel = Self::CorticalPotentialQuant::QUANTIZATION_LEVEL;

    /// Defines the quantization of the membrane potential of a neuron, which all models must
    /// include. This may vary between cortical areas, even of the same model
    type CorticalPotentialQuant: CorticalPotentialQuantization;
}

//endregion
