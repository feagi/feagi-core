use crate::basis::cortical_area::cortical_area_container::{CorticalAreaCachingStrategy, CorticalAreaCachingEnabled, CorticalAreaCachingDisabled};


/// What / if data is cached at runtime
pub trait GraphCaching {
    type CorticalCaching: CorticalAreaCachingStrategy;
}

/// No caching in this graph type
pub struct GraphCachingDisabled;

impl GraphCaching for GraphCachingDisabled {
    type CorticalCaching = CorticalAreaCachingDisabled;
}


pub struct GraphCachingEnabled;

impl GraphCaching for GraphCachingEnabled {
    type CorticalCaching = CorticalAreaCachingEnabled;
}

