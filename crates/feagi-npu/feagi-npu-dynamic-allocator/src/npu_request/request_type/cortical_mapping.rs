
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPUCorticalMappingRequestType {
    AppendMappingEntry,
    RemoveMappingEntry,
    MassEditCorticalMapping,
    RemoveAllMappingsToAndFromArea,
}
