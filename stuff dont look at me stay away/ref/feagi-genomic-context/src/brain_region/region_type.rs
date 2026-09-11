use serde::{Deserialize, Serialize};

// TODO wtf is this. Moved here for now but we should probably just redo it

/// Type of brain region (placeholder for future functional/anatomical classification)
///
/// Currently, no specific region types are defined. This enum serves as a placeholder
/// for future extensions when functional or anatomical classification is implemented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RegionType {
    /// Generic/undefined region type (placeholder)
    #[default]
    Undefined,
}

impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "undefined")
    }
}
