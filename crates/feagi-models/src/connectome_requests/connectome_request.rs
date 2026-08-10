use crate::cortical_area::genome_compose::cortical_writer_by_model_quant::CorticalWriterByModelQuant;
use crate::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::CorticalMappingEntryWriterByModelQuant;
use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
use feagi_genomic_context::cortical_area::CorticalID;

pub enum ConnectomeRequest {
    CorticalAreaAdd {
        TEMP_adding_id: CorticalID,
        writer: CorticalWriterByModelQuant,
    }, // TODO we really shouldnt be taking in a new cortical ID for this
    // CorticalAreaEdit
    // CorticalAreaDelete
    CorticalMappingEntryAdd {
        source_id: CorticalID,
        destination_id: CorticalID,
        mapping_writer: CorticalMappingEntryWriterByModelQuant,
    },
    // TODO BDU cannot handle mapping IDs for now, so only expose an option to delete ALL mappings between 2 cortical areas
}

// TODO this should be a higher level, will be handled by future 'BDU'
/*
/// What effects on the connectome would the request have?
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NPURequestConnectomeConsequences {
    // Ordered in order of operation that they would need to be completed
    /// Deletion of all Cortical Mapping Entries between 2 cortical areas (often bidirectional)
    DeletionOfCorticalMappings,
    /// Deletion of Genome Debugging Interfaces (for cortical area or synapses)
    DeletionOfGenomeDebuggers,
    /// Deletion of Cortical Areas
    DeletionOfCorticalAreas,
    /// Resizing of Cortical areas (their neuron count, or by changing certain properties like their neuron model which will trigger a reallocation)
    ResizingOfCorticalAreas,
    /// Resizing of any mapping entries (often as a result of cortical area sizes changing or connectivity rule changing, but also from changing synapse model type)
    ResizingOfMappingEntries,
    /// Adding new cortical areas
    InsertionOfCorticalAreas,
    /// Adding new cortical mapping (entries)
    InsertionOfCorticalMappingEntries,
    /// Adding any new form of debugging interface
    InsertionOfGenomeDebuggers,
    /// Any in place value update that does not require memory reallocation. Should come last
    InPlaceValueUpdates,
    /// No consequence to the connectome at all (cosmetic change only)
    NoConnectomeChange,
}

 */
