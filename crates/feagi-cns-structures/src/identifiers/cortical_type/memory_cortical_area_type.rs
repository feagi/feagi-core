

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum MemoryCorticalType {
    #[default]
    Memory,
}

impl core::fmt::Display for MemoryCorticalType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Memory => write!(f, "Memory"),
        }
    }
}