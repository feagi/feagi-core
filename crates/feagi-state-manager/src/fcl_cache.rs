// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! FCL (Fire Candidate List) window size cache
//!
//! Stores per-cortical-area FCL window sizes for the burst engine

#[cfg(feature = "std")]
use ahash::AHashMap;

#[cfg(feature = "no_std")]
use heapless::FnvIndexMap as AHashMap;

// Platform-specific imports
#[cfg(all(feature = "std", not(target_family = "wasm")))]
use parking_lot::RwLock;

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
use spin::RwLock;

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
use std::cell::RefCell;

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
use wasm_sync::Mutex;

// ===== Platform-Specific Implementations =====

/// FCL window cache for std platforms
#[cfg(all(feature = "std", not(target_family = "wasm")))]
pub struct FCLWindowCache {
    cache: RwLock<AHashMap<u32, usize>>,
    default_window_size: usize,
}

#[cfg(all(feature = "std", not(target_family = "wasm")))]
impl FCLWindowCache {
    pub fn new(default_window_size: usize) -> Self {
        Self {
            cache: RwLock::new(AHashMap::new()),
            default_window_size,
        }
    }
    
    pub fn get(&self, cortical_area: u32) -> usize {
        let cache = self.cache.read();
        cache.get(&cortical_area).copied().unwrap_or(self.default_window_size)
    }
    
    pub fn set(&self, cortical_area: u32, window_size: usize) {
        let mut cache = self.cache.write();
        cache.insert(cortical_area, window_size);
    }
    
    pub fn remove(&self, cortical_area: u32) {
        let mut cache = self.cache.write();
        cache.remove(&cortical_area);
    }
    
    pub fn get_all(&self) -> AHashMap<u32, usize> {
        let cache = self.cache.read();
        cache.clone()
    }
}

/// FCL window cache for no_std platforms
#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
pub struct FCLWindowCache {
    cache: RwLock<AHashMap<u32, usize>>,
    default_window_size: usize,
}

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
impl FCLWindowCache {
    pub fn new(default_window_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::default()),
            default_window_size,
        }
    }
    
    pub fn get(&self, cortical_area: u32) -> usize {
        let cache = self.cache.read();
        cache.get(&cortical_area).copied().unwrap_or(self.default_window_size)
    }
    
    pub fn set(&self, cortical_area: u32, window_size: usize) {
        let mut cache = self.cache.write();
        cache.insert(cortical_area, window_size);
    }
    
    pub fn remove(&self, cortical_area: u32) {
        let mut cache = self.cache.write();
        cache.remove(&cortical_area);
    }
    
    pub fn get_all(&self) -> AHashMap<u32, usize> {
        let cache = self.cache.read();
        cache.clone()
    }
}

/// FCL window cache for single-threaded WASM
#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
pub struct FCLWindowCache {
    cache: RefCell<AHashMap<u32, usize>>,
    default_window_size: usize,
}

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
impl FCLWindowCache {
    pub fn new(default_window_size: usize) -> Self {
        Self {
            cache: RefCell::new(AHashMap::new()),
            default_window_size,
        }
    }
    
    pub fn get(&self, cortical_area: u32) -> usize {
        let cache = self.cache.borrow();
        cache.get(&cortical_area).copied().unwrap_or(self.default_window_size)
    }
    
    pub fn set(&self, cortical_area: u32, window_size: usize) {
        let mut cache = self.cache.borrow_mut();
        cache.insert(cortical_area, window_size);
    }
    
    pub fn remove(&self, cortical_area: u32) {
        let mut cache = self.cache.borrow_mut();
        cache.remove(&cortical_area);
    }
    
    pub fn get_all(&self) -> AHashMap<u32, usize> {
        let cache = self.cache.borrow();
        cache.clone()
    }
}

/// FCL window cache for multi-threaded WASM
#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
pub struct FCLWindowCache {
    cache: Mutex<AHashMap<u32, usize>>,
    default_window_size: usize,
}

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
impl FCLWindowCache {
    pub fn new(default_window_size: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            default_window_size,
        }
    }
    
    pub fn get(&self, cortical_area: u32) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.get(&cortical_area).copied().unwrap_or(self.default_window_size)
    }
    
    pub fn set(&self, cortical_area: u32, window_size: usize) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(cortical_area, window_size);
    }
    
    pub fn remove(&self, cortical_area: u32) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(&cortical_area);
    }
    
    pub fn get_all(&self) -> AHashMap<u32, usize> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_window_size() {
        let cache = FCLWindowCache::new(20);
        assert_eq!(cache.get(0), 20);
    }
    
    #[test]
    fn test_set_and_get() {
        let cache = FCLWindowCache::new(20);
        
        cache.set(0, 10);
        assert_eq!(cache.get(0), 10);
        
        cache.set(1, 30);
        assert_eq!(cache.get(1), 30);
    }
    
    #[test]
    fn test_remove() {
        let cache = FCLWindowCache::new(20);
        
        cache.set(0, 10);
        assert_eq!(cache.get(0), 10);
        
        cache.remove(0);
        assert_eq!(cache.get(0), 20); // Back to default
    }
}
