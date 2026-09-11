use serde::{Deserialize, Serialize};

/// Index for a stage / stage property within a pipeline.
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct PipelineStagePropertyIndex(u32);

impl PipelineStagePropertyIndex {
    pub const fn from(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl std::ops::Deref for PipelineStagePropertyIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for PipelineStagePropertyIndex {
    fn from(value: u32) -> Self {
        PipelineStagePropertyIndex(value)
    }
}

impl From<PipelineStagePropertyIndex> for u32 {
    fn from(value: PipelineStagePropertyIndex) -> Self {
        value.0
    }
}

impl std::fmt::Display for PipelineStagePropertyIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
