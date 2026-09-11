/// Identifies a specific cortical mapping entry within the context of a cortical mapping (NOT global)
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalMappingEntryID(u32);

// TODO can we encode specific context about the type of mapping within this ID itself?
