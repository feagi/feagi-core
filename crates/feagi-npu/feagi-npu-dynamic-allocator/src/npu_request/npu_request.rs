/// Defines any single npu edit request to be interpreted by the NPU and if valid, executed upon
pub trait ConnectomeRequest: Clone {
    /// How will this request impact the connectome?
    fn get_connectome_consequences() -> NPURequestConnectomeConsequences;
}

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


