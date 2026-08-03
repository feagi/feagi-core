use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
use feagi_genomic_context::cortical_area::CorticalID;

pub struct ConnectomeRequest {
    pub index_level: FeagiIndexQuantizationLevel,
    pub request_type: ConnectomeRequestType,

}

// TODO define model and quantization
pub enum ConnectomeRequestType {
    CorticalAreaAddDimensional(CorticalID, DimensionalCorticalArea4DDimensions<u64>),
    CorticalAreaAddFormless(CorticalID, u64),
    CorticalAreaDelete(CorticalID),
//    MappingEntryAdd(CorticalID, CorticalID, )
    
}


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