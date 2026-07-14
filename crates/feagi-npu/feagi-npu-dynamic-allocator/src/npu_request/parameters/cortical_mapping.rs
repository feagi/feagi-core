use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;

/// Add / Remove / Edit Cortical Mappings
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersCorticalMapping {
    AppendMappingEntry{
        source: CorticalID,
        destination: CorticalID,
        new_mapping_entry: (),
    }
}

