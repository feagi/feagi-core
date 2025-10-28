//! State persistence (save/load to disk)

#[cfg(feature = "std")]
use crate::{StateError, Result};

#[cfg(feature = "std")]
pub fn save_state(path: &std::path::Path) -> Result<()> {
    // TODO: Implement
    Ok(())
}

#[cfg(feature = "std")]
pub fn load_state(path: &std::path::Path) -> Result<()> {
    // TODO: Implement
    Ok(())
}

