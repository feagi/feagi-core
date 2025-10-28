//! FCL (Fire Candidate List) window size cache

/// FCL window size cache
pub struct FCLWindowCache {
    // TODO: Implement
    _phantom: std::marker::PhantomData<()>,
}

impl FCLWindowCache {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for FCLWindowCache {
    fn default() -> Self {
        Self::new()
    }
}

