use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizationLevelPacking};
use feagi_models::cortical_mapping_entry::synapse_model_implementations::generated_enums::SynapseModelTypeAndQuantizationPacked;
use feagi_models::wrapped_index_collections::{MappingEntryModelIndex, SynapseEngineIndex};
use crate::npu::burst_engine::common_cpu_structs::flags::cortical_mapping_entry_runtime_flags::CorticalMappingEntryRuntimeFlags;

/// For a cortical mapping entry, contains indexes for some corresponding properties belonging to it
#[derive(Clone, Copy)]
pub struct CorticalMappingEntryIndexLookupTable<FIQ: FeagiIndexQuantization> {
    pub mapping_entry_model_index: MappingEntryModelIndex<FIQ::CorticalMappingEntryIndexCountQuant>,
}

/// Holds various universal properties of all Cortical Mapping Entries
#[derive(Clone, Copy)]
pub struct CorticalMappingEntryProperties {
    pub flags: CorticalMappingEntryRuntimeFlags,
    pub model_and_quant: SynapseModelTypeAndQuantizationPacked,
    pub source_destination_mp_quants: SynapseMappingMPQuants,
    /// Delay in bursts between the source firing and the potential arriving. Matches the width the
    /// models layer expresses it at, so no narrowing happens at the genome boundary.
    pub delay: u16,
}

/// For a cortical mapping entry, contains indexes for some corresponding properties belonging to it
#[derive(Clone, Copy)]
pub struct SynapseIndexLookupTable<FIQ: FeagiIndexQuantization> {
    pub synapse_first_local_index: SynapseEngineIndex<FIQ::SynapseIndexCountQuant>,

    pub mapping_entry_model_index: MappingEntryModelIndex<FIQ::CorticalMappingEntryIndexCountQuant>,
}

///
#[derive(Clone, Copy)]
pub struct SynapseMappingMPQuants(u8);

impl SynapseMappingMPQuants {
    pub fn new(source_mp: DecimalQuantizationLevel, destination_mp: DecimalQuantizationLevel) -> Self {
        Self(((source_mp as u8) << DecimalQuantizationLevel::NUMBER_BITS) | destination_mp as u8)
    }

    pub fn source_mp(&self) -> DecimalQuantizationLevel {
        // The shift already discards the destination's bits, leaving only the source's.
        unsafe { DecimalQuantizationLevel::from_unpacked_byte(self.0 >> DecimalQuantizationLevel::NUMBER_BITS) }
    }

    pub fn destination_mp(&self) -> DecimalQuantizationLevel {
        // Masked first: the source occupies the high nibble, and transmuting the packed byte whole
        // would build the enum out of a value no variant has.
        let mut byte = self.0;
        DecimalQuantizationLevel::apply_mask(&mut byte);
        unsafe { DecimalQuantizationLevel::from_unpacked_byte(byte) }
    }
}
