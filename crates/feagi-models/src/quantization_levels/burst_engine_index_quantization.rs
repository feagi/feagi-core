use core::hash::Hash;
use feagi_data::values::quantizable::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
// TODO xxhash?


pub trait BurstEngineIndexQuantization: Clone + Copy + Hash + PartialEq + Eq + Sync + 'static  {
    const LEVEL: BurstEngineIndexQuantizationLevel;

    /// Cortical area indexing within the burst engine
    type CorticalAreaIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Defines all neuron indexing (linear, voxel, etc) within the burst engine
    type NeuronIndexQuant: QuantizedUnsignedIntegerUnwrappedTrait;
    
    /// Indexing of cortical mapping entries within the burst engine.
    type CorticalMappingEntryIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;

    /// Indexing of synapses within the burst engine
    type SynapseIndexCountQuant: QuantizedUnsignedIntegerUnwrappedTrait;
}

/// The quantization level that is for structures that must be of the same quantization across
/// all burst engines in the Neural Processing Unit as an Enum
#[repr(u8)]
#[derive(Default, Debug, Clone, PartialEq)]
pub enum BurstEngineIndexQuantizationLevel {
    
    #[default]
    /// Most Genomes can fit in this fine
    Normal = 0,

    /// Mainly only useful for low end devices with small genomes
    Small = 1,
    
    /// Can fit larger Genomes, assuming you have the memory...
    Large = 2,

    /// Everything uses the max index. Not very practical for computation but can be used for sending / storing data without worrying about limits
    MaxForGenome = 3,
}

/// Most Genomes can fit in this fine
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BurstEngineIndexQuantizationQuantizationNormal;

impl BurstEngineIndexQuantization for  BurstEngineIndexQuantizationQuantizationNormal{
    const LEVEL: BurstEngineIndexQuantizationLevel = BurstEngineIndexQuantizationLevel::Normal;
    type CorticalAreaIndexCountQuant = u16;
    type NeuronIndexQuant = u32;
    type CorticalMappingEntryIndexCountQuant = u16;
    type SynapseIndexCountQuant = u32;
}



/// Mainly only useful for low end devices with small genomes
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BurstEngineIndexQuantizationQuantizationSmall;

impl BurstEngineIndexQuantization for  BurstEngineIndexQuantizationQuantizationSmall{
    const LEVEL: BurstEngineIndexQuantizationLevel = BurstEngineIndexQuantizationLevel::Small;
    type CorticalAreaIndexCountQuant = u8;
    type NeuronIndexQuant = u16;
    type CorticalMappingEntryIndexCountQuant = u8;
    type SynapseIndexCountQuant = u16;
}



/// Can fit larger Genomes, assuming you have the memory...
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BurstEngineIndexQuantizationQuantizationLarge;

impl BurstEngineIndexQuantization for  BurstEngineIndexQuantizationQuantizationLarge{
    const LEVEL: BurstEngineIndexQuantizationLevel = BurstEngineIndexQuantizationLevel::Large;
    type CorticalAreaIndexCountQuant = u32;
    type NeuronIndexQuant = u64;
    type CorticalMappingEntryIndexCountQuant = u32;
    type SynapseIndexCountQuant = u64;
}


/// Everything uses the max index. Not very practical for computation but can be used for sending / storing data without worrying about limits
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BurstEngineIndexQuantizationQuantizationMaxForGenome;

impl BurstEngineIndexQuantization for  BurstEngineIndexQuantizationQuantizationMaxForGenome{
    const LEVEL: BurstEngineIndexQuantizationLevel = BurstEngineIndexQuantizationLevel::MaxForGenome;
    type CorticalAreaIndexCountQuant = u64;
    type NeuronIndexQuant = u64;
    type CorticalMappingEntryIndexCountQuant = u64;
    type SynapseIndexCountQuant = u64;
}
