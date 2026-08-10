// TODO this should be macro generated

use crate::cortical_mapping_entry::components::doublet::doublet_iterator_type::DoubletIteratorDimensionalTypeGenomic;

/// This enum defines what cortical mapping entry will be created and how. Different Synapse
/// models have different instantiation parameters and quantization levels, and this handles that
pub enum CorticalMappingEntryWriterByModelQuant {
    Uniform(UniformWriter),
}

// TODO a bit rushed right now, needs some more work as this is a temp solution
pub enum UniformWriter {
    Standard(DoubletIteratorDimensionalTypeGenomic),
}
