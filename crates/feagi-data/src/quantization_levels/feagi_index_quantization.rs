// TODO Rename to FeagiIndexQuantization!
//! Sets the indexing of various data types, where higher quantizations can support bigger
//! collections of those items but at an increased memory cost

use crate::values::quantizable::QuantizedIndexCountTrait;

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

    /// Indexing of cortical_area areas within the NPU. Note that indexes are not stable outside the NPU!
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


//endregion


#[repr(u8)]
#[derive(Default)]
pub enum FeagiGlobalQuantizationLevel {
    #[default]
    Standard = 0,
    // TODO tiny, mini, big, absurd
}
