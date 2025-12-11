// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Cortical area locking for concurrent operations
//!
//! Allows locking specific cortical areas for neurogenesis or plasticity
//! operations without blocking the entire brain.

#[cfg(feature = "std")]
use ahash::AHashSet;

#[cfg(feature = "no_std")]
use heapless::FnvIndexSet as AHashSet;

// Platform-specific imports
#[cfg(all(feature = "std", not(target_family = "wasm")))]
use parking_lot::Mutex;

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
use spin::Mutex;

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
use std::cell::RefCell;

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
use wasm_sync::Mutex;

// ===== Platform-Specific Implementations =====

/// Cortical lock manager for std platforms
#[cfg(all(feature = "std", not(target_family = "wasm")))]
pub struct CorticalLockManager {
    locked_areas: Mutex<AHashSet<u32>>,
}

#[cfg(all(feature = "std", not(target_family = "wasm")))]
impl CorticalLockManager {
    pub fn new() -> Self {
        Self {
            locked_areas: Mutex::new(AHashSet::new()),
        }
    }
    
    /// Try to lock a cortical area (returns true if successful)
    pub fn try_lock(&self, cortical_area: u32) -> bool {
        let mut locked = self.locked_areas.lock();
        locked.insert(cortical_area)
    }
    
    /// Unlock a cortical area
    pub fn unlock(&self, cortical_area: u32) {
        let mut locked = self.locked_areas.lock();
        locked.remove(&cortical_area);
    }
    
    /// Check if a cortical area is locked
    pub fn is_locked(&self, cortical_area: u32) -> bool {
        let locked = self.locked_areas.lock();
        locked.contains(&cortical_area)
    }
    
    /// Get all locked areas
    pub fn get_locked_areas(&self) -> Vec<u32> {
        let locked = self.locked_areas.lock();
        locked.iter().copied().collect()
    }
}

/// Cortical lock manager for no_std platforms
#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
pub struct CorticalLockManager {
    locked_areas: Mutex<AHashSet<u32>>,
}

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
impl CorticalLockManager {
    pub fn new() -> Self {
        Self {
            locked_areas: Mutex::new(AHashSet::default()),
        }
    }
    
    pub fn try_lock(&self, cortical_area: u32) -> bool {
        let mut locked = self.locked_areas.lock();
        locked.insert(cortical_area)
    }
    
    pub fn unlock(&self, cortical_area: u32) {
        let mut locked = self.locked_areas.lock();
        locked.remove(&cortical_area);
    }
    
    pub fn is_locked(&self, cortical_area: u32) -> bool {
        let locked = self.locked_areas.lock();
        locked.contains(&cortical_area)
    }
    
    pub fn get_locked_areas(&self) -> Vec<u32> {
        let locked = self.locked_areas.lock();
        locked.iter().copied().collect()
    }
}

/// Cortical lock manager for single-threaded WASM
#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
pub struct CorticalLockManager {
    locked_areas: RefCell<AHashSet<u32>>,
}

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
impl CorticalLockManager {
    pub fn new() -> Self {
        Self {
            locked_areas: RefCell::new(AHashSet::new()),
        }
    }
    
    pub fn try_lock(&self, cortical_area: u32) -> bool {
        let mut locked = self.locked_areas.borrow_mut();
        locked.insert(cortical_area)
    }
    
    pub fn unlock(&self, cortical_area: u32) {
        let mut locked = self.locked_areas.borrow_mut();
        locked.remove(&cortical_area);
    }
    
    pub fn is_locked(&self, cortical_area: u32) -> bool {
        let locked = self.locked_areas.borrow();
        locked.contains(&cortical_area)
    }
    
    pub fn get_locked_areas(&self) -> Vec<u32> {
        let locked = self.locked_areas.borrow();
        locked.iter().copied().collect()
    }
}

/// Cortical lock manager for multi-threaded WASM
#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
pub struct CorticalLockManager {
    locked_areas: Mutex<AHashSet<u32>>,
}

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
impl CorticalLockManager {
    pub fn new() -> Self {
        Self {
            locked_areas: Mutex::new(AHashSet::new()),
        }
    }
    
    pub fn try_lock(&self, cortical_area: u32) -> bool {
        let mut locked = self.locked_areas.lock().unwrap();
        locked.insert(cortical_area)
    }
    
    pub fn unlock(&self, cortical_area: u32) {
        let mut locked = self.locked_areas.lock().unwrap();
        locked.remove(&cortical_area);
    }
    
    pub fn is_locked(&self, cortical_area: u32) -> bool {
        let locked = self.locked_areas.lock().unwrap();
        locked.contains(&cortical_area)
    }
    
    pub fn get_locked_areas(&self) -> Vec<u32> {
        let locked = self.locked_areas.lock().unwrap();
        locked.iter().copied().collect()
    }
}

impl Default for CorticalLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lock_unlock() {
        let manager = CorticalLockManager::new();
        
        assert!(manager.try_lock(0));
        assert!(manager.is_locked(0));
        
        manager.unlock(0);
        assert!(!manager.is_locked(0));
    }
    
    #[test]
    fn test_duplicate_lock() {
        let manager = CorticalLockManager::new();
        
        assert!(manager.try_lock(0));
        assert!(!manager.try_lock(0)); // Already locked
    }
    
    #[test]
    fn test_multiple_areas() {
        let manager = CorticalLockManager::new();
        
        manager.try_lock(0);
        manager.try_lock(1);
        manager.try_lock(2);
        
        let locked = manager.get_locked_areas();
        assert_eq!(locked.len(), 3);
    }
}
