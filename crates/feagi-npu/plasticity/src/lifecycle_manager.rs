// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Plasticity Service Lifecycle Manager
//!
//! Dynamically starts/stops the plasticity service based on genome configuration.
//! - Only runs when memory areas are present in the genome
//! - Automatically starts when memory areas are added
//! - Automatically stops when all memory areas are removed
//! - Processes commands and applies them to the NPU

use crate::memory_stats_cache::MemoryStatsCache;
use crate::service::{PlasticityCommand, PlasticityConfig, PlasticityService};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, info, warn};

/// Plasticity lifecycle manager
/// Manages the plasticity service lifecycle and command processing
pub struct PlasticityLifecycleManager {
    /// The plasticity service (if running)
    service: Arc<Mutex<Option<PlasticityService>>>,
    
    /// Thread handle for the plasticity worker (if running)
    thread_handle: Option<thread::JoinHandle<()>>,
    
    /// Running flag
    running: Arc<AtomicBool>,
    
    /// Configuration for the plasticity service
    config: PlasticityConfig,
    
    /// Memory stats cache (shared with health check)
    memory_stats_cache: MemoryStatsCache,
    
    /// Count of registered memory areas
    memory_area_count: Arc<RwLock<usize>>,
}

impl PlasticityLifecycleManager {
    /// Create a new plasticity lifecycle manager
    pub fn new(config: PlasticityConfig, memory_stats_cache: MemoryStatsCache) -> Self {
        Self {
            service: Arc::new(Mutex::new(None)),
            thread_handle: None,
            running: Arc::new(AtomicBool::new(false)),
            config,
            memory_stats_cache,
            memory_area_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if the plasticity service is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the memory stats cache for wiring to the health check
    pub fn get_memory_stats_cache(&self) -> MemoryStatsCache {
        self.memory_stats_cache.clone()
    }

    /// Start the plasticity service if not already running
    /// Returns true if started, false if already running
    pub fn start_if_needed(&mut self) -> bool {
        if self.is_running() {
            return false;
        }

        info!("Starting plasticity service");
        
        let mut service_lock = self.service.lock().unwrap();
        let service = PlasticityService::new(self.config.clone(), self.memory_stats_cache.clone());
        
        // Start the service thread
        let thread_handle = service.start();
        
        // Mark as running
        self.running.store(true, Ordering::Relaxed);
        
        *service_lock = Some(service);
        drop(service_lock);
        
        self.thread_handle = Some(thread_handle);
        
        info!("Plasticity service started");
        true
    }

    /// Stop the plasticity service if running
    /// Returns true if stopped, false if not running
    pub fn stop_if_needed(&mut self) -> bool {
        if !self.is_running() {
            return false;
        }

        info!("Stopping plasticity service");
        
        let mut service_lock = self.service.lock().unwrap();
        if let Some(service) = service_lock.take() {
            service.stop();
        }
        drop(service_lock);
        
        // Mark as stopped
        self.running.store(false, Ordering::Relaxed);
        
        // Join the thread (this will wait for it to finish)
        if let Some(handle) = self.thread_handle.take() {
            // Note: The thread should exit after stop() is called
            // If it doesn't, this will block. In production, consider a timeout.
            let _ = handle.join();
        }
        
        info!("Plasticity service stopped");
        true
    }

    /// Register a memory area (increments count, starts service if needed)
    pub fn register_memory_area(
        &mut self,
        area_idx: u32,
        area_name: String,
        temporal_depth: u32,
        upstream_areas: Vec<u32>,
        lifecycle_config: Option<crate::memory_neuron_array::MemoryNeuronLifecycleConfig>,
    ) {
        // Increment count
        {
            let mut count = self.memory_area_count.write();
            *count += 1;
            debug!("Memory area registered: {} (total: {})", area_name, *count);
        }

        // Start service if this is the first memory area
        if !self.is_running() {
            self.start_if_needed();
        }

        // Register with the service
        if let Some(service) = self.service.lock().unwrap().as_ref() {
            service.register_memory_area(area_idx, area_name, temporal_depth, upstream_areas, lifecycle_config);
        }
    }

    /// Unregister a memory area (decrements count, stops service if count reaches 0)
    pub fn unregister_memory_area(&mut self) {
        let should_stop = {
            let mut count = self.memory_area_count.write();
            if *count > 0 {
                *count -= 1;
            }
            debug!("Memory area unregistered (remaining: {})", *count);
            *count == 0
        };

        // Stop service if no more memory areas
        if should_stop && self.is_running() {
            self.stop_if_needed();
        }
    }

    /// Notify the plasticity service of a new burst
    pub fn notify_burst(&self, timestep: u64) {
        if let Some(service) = self.service.lock().unwrap().as_ref() {
            service.notify_burst(timestep);
        }
    }

    /// Drain commands from the plasticity service and return them for processing
    /// This should be called after each burst by the burst loop runner
    pub fn drain_commands(&self) -> Vec<PlasticityCommand> {
        if let Some(service) = self.service.lock().unwrap().as_ref() {
            service.drain_commands()
        } else {
            Vec::new()
        }
    }
}

impl Drop for PlasticityLifecycleManager {
    fn drop(&mut self) {
        // Ensure service is stopped when manager is dropped
        if self.is_running() {
            warn!("PlasticityLifecycleManager dropped while service was running, stopping now");
            self.stop_if_needed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_start_stop() {
        let cache = crate::memory_stats_cache::create_memory_stats_cache();
        let config = PlasticityConfig::default();
        let mut manager = PlasticityLifecycleManager::new(config, cache);

        assert!(!manager.is_running());

        // Start
        assert!(manager.start_if_needed());
        assert!(manager.is_running());

        // Try to start again (should return false)
        assert!(!manager.start_if_needed());

        // Stop
        assert!(manager.stop_if_needed());
        assert!(!manager.is_running());

        // Try to stop again (should return false)
        assert!(!manager.stop_if_needed());
    }

    #[test]
    fn test_auto_start_on_register() {
        let cache = crate::memory_stats_cache::create_memory_stats_cache();
        let config = PlasticityConfig::default();
        let mut manager = PlasticityLifecycleManager::new(config, cache);

        assert!(!manager.is_running());

        // Register a memory area (should auto-start)
        manager.register_memory_area(0, "mem_00".to_string(), 3, vec![1, 2], None);
        
        assert!(manager.is_running());

        // Clean up
        manager.stop_if_needed();
    }
}

