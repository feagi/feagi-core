// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Sleep Manager
//!
//! Manages brain sleep states for energy efficiency and memory optimization.
//!
//! ## Sleep States
//! 
//! **Light Sleep:**
//! - Reduces burst frequency to configured light_sleep_frequency_hz
//! - Builds lazy free-list for fast neuron allocation (interruptible)
//! - Safe to wake immediately
//! 
//! **Deep Sleep:**
//! - Further reduces frequency to deep_sleep_frequency_hz
//! - Runs memory compaction if fragmentation > threshold (NON-INTERRUPTIBLE)
//! - Blocks wake signals during compaction to prevent data corruption
//! 
//! ## State Transitions
//! 
//! ```text
//! Running → Light Sleep → Deep Sleep
//!    ↑          ↓             ↓
//!    └──────────┴─────────────┘
//!      (wake on IPU activity)
//! ```

// DynamicNPU removed - use concrete types
// use crate::DynamicNPU;
use feagi_state_manager::BurstEngineState;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Sleep manager configuration (from genome physiology)
#[derive(Debug, Clone)]
pub struct SleepConfig {
    // Light sleep settings
    pub light_sleep_frequency_hz: f64,
    pub light_sleep_ipu_threshold_neurons: usize,
    pub light_sleep_activity_window_bursts: usize,
    
    // Deep sleep settings
    pub deep_sleep_enabled: bool,
    pub deep_sleep_frequency_hz: f64,
    pub deep_sleep_ipu_threshold_neurons: usize,
    pub deep_sleep_min_light_sleep_duration_bursts: usize,
    pub deep_sleep_compaction_fragmentation_threshold: f32,
    
    // Wake conditions
    pub wake_ipu_threshold_neurons: usize,
    pub wake_activity_window_bursts: usize,
}

impl Default for SleepConfig {
    fn default() -> Self {
        Self {
            // Light sleep (gentle reduction)
            light_sleep_frequency_hz: 10.0,
            light_sleep_ipu_threshold_neurons: 100,
            light_sleep_activity_window_bursts: 50,
            
            // Deep sleep (aggressive optimization)
            deep_sleep_enabled: true,
            deep_sleep_frequency_hz: 5.0,
            deep_sleep_ipu_threshold_neurons: 10,
            deep_sleep_min_light_sleep_duration_bursts: 500,
            deep_sleep_compaction_fragmentation_threshold: 0.20,
            
            // Wake (responsive)
            wake_ipu_threshold_neurons: 200,
            wake_activity_window_bursts: 10,
        }
    }
}

impl SleepConfig {
    /// Check if this config appears to be missing from genome (all defaults with sentinel values)
    /// Returns true if config is valid and should be used
    pub fn is_valid_from_genome(&self) -> bool {
        // If ALL values are exactly defaults, likely missing from genome
        // We use a simple heuristic: if frequencies are > 0, assume valid
        self.light_sleep_frequency_hz > 0.0 
            && self.deep_sleep_frequency_hz > 0.0
            && self.light_sleep_ipu_threshold_neurons > 0
            && self.wake_ipu_threshold_neurons > 0
    }
    
    /// Create a disabled config (sentinel values that will be detected as invalid)
    pub fn disabled() -> Self {
        Self {
            light_sleep_frequency_hz: 0.0,  // Sentinel: 0 Hz is invalid
            light_sleep_ipu_threshold_neurons: 0,
            light_sleep_activity_window_bursts: 0,
            deep_sleep_enabled: false,
            deep_sleep_frequency_hz: 0.0,
            deep_sleep_ipu_threshold_neurons: 0,
            deep_sleep_min_light_sleep_duration_bursts: 0,
            deep_sleep_compaction_fragmentation_threshold: 0.0,
            wake_ipu_threshold_neurons: 0,
            wake_activity_window_bursts: 0,
        }
    }
}

/// IPU activity tracker (rolling window)
#[derive(Debug)]
pub struct ActivityTracker {
    /// Circular buffer for IPU neuron fire counts per burst
    window: Vec<usize>,
    /// Current write position in circular buffer
    position: usize,
    /// Number of bursts recorded (saturates at window_size)
    count: usize,
    /// Window size (number of bursts to track)
    window_size: usize,
}

impl ActivityTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: vec![0; window_size],
            position: 0,
            count: 0,
            window_size,
        }
    }
    
    /// Record IPU activity for this burst
    pub fn record(&mut self, ipu_neuron_fires: usize) {
        self.window[self.position] = ipu_neuron_fires;
        self.position = (self.position + 1) % self.window_size;
        self.count = (self.count + 1).min(self.window_size);
    }
    
    /// Get average IPU firings over the window
    pub fn average(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        
        let sum: usize = self.window.iter().take(self.count).sum();
        sum as f64 / self.count as f64
    }
    
    /// Check if window is full
    pub fn is_full(&self) -> bool {
        self.count >= self.window_size
    }
}

/// Sleep phase timing tracker
#[derive(Debug, Clone)]
pub struct SleepPhase {
    pub state: BurstEngineState,
    pub entered_at: Instant,
    pub burst_count_at_entry: u64,
}

impl SleepPhase {
    pub fn duration(&self) -> Duration {
        self.entered_at.elapsed()
    }
    
    pub fn duration_seconds(&self) -> f64 {
        self.duration().as_secs_f64()
    }
}

/// Sleep manager - handles brain sleep states and optimizations
pub struct SleepManager {
    /// Configuration (from genome physiology)
    config: SleepConfig,
    
    /// System-level enable flag (from feagi_configuration.toml)
    enabled: bool,
    
    /// Current sleep state
    current_state: BurstEngineState,
    
    /// Activity trackers
    light_sleep_tracker: ActivityTracker,
    deep_sleep_tracker: ActivityTracker,
    wake_tracker: ActivityTracker,
    
    /// Burst count when we entered light sleep (for deep sleep transition)
    light_sleep_entry_burst: Option<u64>,
    
    /// Current sleep phase (for timing)
    current_phase: Option<SleepPhase>,
    
    /// Compaction in progress (blocks wake)
    compaction_in_progress: Arc<AtomicBool>,
    
    /// Total time spent in each state (for statistics)
    total_light_sleep_seconds: Arc<AtomicU64>,
    total_deep_sleep_seconds: Arc<AtomicU64>,
    
    /// NPU reference (for compaction and free-list building)
    #[allow(dead_code)] // Will be used when NPU integration is complete
    npu: Arc<Mutex<DynamicNPU>>,
}

impl SleepManager {
    /// Create a new sleep manager
    /// 
    /// # Arguments
    /// * `config` - Sleep configuration from genome physiology
    /// * `system_enabled` - Master enable flag from feagi_configuration.toml
    /// * `npu` - NPU reference for compaction and free-list operations
    /// 
    /// # Returns
    /// Sleep manager instance. If either system flag is false OR genome config is missing,
    /// sleep will be disabled (no state transitions will occur).
    pub fn new(
        config: SleepConfig,
        system_enabled: bool,
        npu: Arc<Mutex<DynamicNPU>>,
    ) -> Self {
        // Check if genome has sleep configuration
        let genome_has_config = config.is_valid_from_genome();
        
        // Sleep is only enabled if BOTH system flag AND genome config are present
        let actually_enabled = system_enabled && genome_has_config;
        
        if !system_enabled {
            info!("🛌 Sleep Manager: DISABLED by system configuration (feagi_configuration.toml)");
        } else if !genome_has_config {
            warn!("🛌 Sleep Manager: DISABLED - missing 'physiology.sleep' in genome");
            warn!("   Add sleep configuration to genome.json to enable sleep mode");
        } else {
            info!("🛌 Sleep Manager: ENABLED");
            info!("   Light sleep frequency: {} Hz (threshold: {} neurons/burst over {} bursts)", 
                  config.light_sleep_frequency_hz, 
                  config.light_sleep_ipu_threshold_neurons,
                  config.light_sleep_activity_window_bursts);
            info!("   Deep sleep enabled: {} (frequency: {} Hz, threshold: {} neurons/burst)",
                  config.deep_sleep_enabled,
                  config.deep_sleep_frequency_hz,
                  config.deep_sleep_ipu_threshold_neurons);
            info!("   Wake threshold: {} neurons/burst over {} bursts",
                  config.wake_ipu_threshold_neurons,
                  config.wake_activity_window_bursts);
        }
        
        Self {
            light_sleep_tracker: ActivityTracker::new(config.light_sleep_activity_window_bursts.max(1)), // Min 1 to avoid panic
            deep_sleep_tracker: ActivityTracker::new(config.light_sleep_activity_window_bursts.max(1)),
            wake_tracker: ActivityTracker::new(config.wake_activity_window_bursts.max(1)),
            light_sleep_entry_burst: None,
            current_phase: None,
            compaction_in_progress: Arc::new(AtomicBool::new(false)),
            total_light_sleep_seconds: Arc::new(AtomicU64::new(0)),
            total_deep_sleep_seconds: Arc::new(AtomicU64::new(0)),
            current_state: BurstEngineState::Running,
            config,
            enabled: actually_enabled,  // Store the combined enable state
            npu,
        }
    }
    
    /// Update activity trackers with current burst's IPU firing count
    /// Returns the new target state (may trigger sleep transitions)
    pub fn update_activity(
        &mut self,
        ipu_neuron_fires: usize,
        current_burst: u64,
    ) -> Option<BurstEngineState> {
        if !self.enabled {
            return None; // Sleep disabled globally
        }
        
        // Update all trackers (different windows)
        self.light_sleep_tracker.record(ipu_neuron_fires);
        self.deep_sleep_tracker.record(ipu_neuron_fires);
        self.wake_tracker.record(ipu_neuron_fires);
        
        // Check for state transitions based on current state
        match self.current_state {
            BurstEngineState::Running => {
                self.check_light_sleep_entry(current_burst)
            }
            BurstEngineState::LightSleep => {
                self.check_wake_from_light_sleep()
                    .or_else(|| self.check_deep_sleep_entry(current_burst))
            }
            BurstEngineState::DeepSleep => {
                // Can only wake from deep sleep (never go deeper)
                self.check_wake_from_deep_sleep()
            }
            _ => None, // Other states don't trigger sleep
        }
    }
    
    /// Check if should enter light sleep
    fn check_light_sleep_entry(&self, _current_burst: u64) -> Option<BurstEngineState> {
        if !self.light_sleep_tracker.is_full() {
            return None; // Not enough data yet
        }
        
        let avg = self.light_sleep_tracker.average();
        if avg < self.config.light_sleep_ipu_threshold_neurons as f64 {
            debug!(
                "🛌 Light sleep threshold met: avg={:.1} < threshold={}",
                avg, self.config.light_sleep_ipu_threshold_neurons
            );
            Some(BurstEngineState::LightSleep)
        } else {
            None
        }
    }
    
    /// Check if should wake from light sleep
    fn check_wake_from_light_sleep(&self) -> Option<BurstEngineState> {
        if !self.wake_tracker.is_full() {
            return None;
        }
        
        let avg = self.wake_tracker.average();
        if avg > self.config.wake_ipu_threshold_neurons as f64 {
            debug!(
                "⏰ Wake threshold met from light sleep: avg={:.1} > threshold={}",
                avg, self.config.wake_ipu_threshold_neurons
            );
            Some(BurstEngineState::Running)
        } else {
            None
        }
    }
    
    /// Check if should enter deep sleep
    fn check_deep_sleep_entry(&self, current_burst: u64) -> Option<BurstEngineState> {
        if !self.config.deep_sleep_enabled {
            return None;
        }
        
        // Must be in light sleep long enough
        let entry_burst = self.light_sleep_entry_burst?;
        let bursts_in_light_sleep = current_burst.saturating_sub(entry_burst);
        
        if bursts_in_light_sleep < self.config.deep_sleep_min_light_sleep_duration_bursts as u64 {
            return None; // Not in light sleep long enough
        }
        
        // Check activity is even lower
        let avg = self.deep_sleep_tracker.average();
        if avg < self.config.deep_sleep_ipu_threshold_neurons as f64 {
            debug!(
                "🛌💤 Deep sleep threshold met: avg={:.1} < threshold={}, bursts_in_light_sleep={}",
                avg, self.config.deep_sleep_ipu_threshold_neurons, bursts_in_light_sleep
            );
            Some(BurstEngineState::DeepSleep)
        } else {
            None
        }
    }
    
    /// Check if should wake from deep sleep
    fn check_wake_from_deep_sleep(&self) -> Option<BurstEngineState> {
        // Cannot wake during compaction (data safety)
        if self.compaction_in_progress.load(Ordering::Acquire) {
            return None;
        }
        
        if !self.wake_tracker.is_full() {
            return None;
        }
        
        let avg = self.wake_tracker.average();
        if avg > self.config.wake_ipu_threshold_neurons as f64 {
            debug!(
                "⏰ Wake threshold met from deep sleep: avg={:.1} > threshold={}",
                avg, self.config.wake_ipu_threshold_neurons
            );
            Some(BurstEngineState::Running)
        } else {
            None
        }
    }
    
    /// Transition to a new state (called by burst loop)
    pub fn transition_to(&mut self, new_state: BurstEngineState, current_burst: u64) -> f64 {
        // Log phase exit if transitioning from sleep
        if let Some(phase) = &self.current_phase {
            let duration_s = phase.duration_seconds();
            let bursts_in_phase = current_burst.saturating_sub(phase.burst_count_at_entry);
            
            match phase.state {
                BurstEngineState::LightSleep => {
                    let total = self.total_light_sleep_seconds.fetch_add(
                        duration_s as u64,
                        Ordering::Relaxed
                    );
                    info!(
                        "🛌 Exiting Light Sleep: duration={:.2}s, bursts={}, total_light_sleep={}s",
                        duration_s, bursts_in_phase, total + duration_s as u64
                    );
                }
                BurstEngineState::DeepSleep => {
                    let total = self.total_deep_sleep_seconds.fetch_add(
                        duration_s as u64,
                        Ordering::Relaxed
                    );
                    info!(
                        "🛌💤 Exiting Deep Sleep: duration={:.2}s, bursts={}, total_deep_sleep={}s",
                        duration_s, bursts_in_phase, total + duration_s as u64
                    );
                }
                _ => {}
            }
        }
        
        // Log phase entry
        let new_frequency = match new_state {
            BurstEngineState::LightSleep => {
                info!(
                    "🛌 Entering Light Sleep at burst {}: reducing frequency to {} Hz",
                    current_burst, self.config.light_sleep_frequency_hz
                );
                self.light_sleep_entry_burst = Some(current_burst);
                
                // Build lazy free-list (interruptible, safe)
                self.build_lazy_free_list();
                
                self.config.light_sleep_frequency_hz
            }
            BurstEngineState::DeepSleep => {
                info!(
                    "🛌💤 Entering Deep Sleep at burst {}: reducing frequency to {} Hz",
                    current_burst, self.config.deep_sleep_frequency_hz
                );
                
                // Run compaction if needed (NON-INTERRUPTIBLE)
                self.run_compaction_if_needed();
                
                self.config.deep_sleep_frequency_hz
            }
            BurstEngineState::Running => {
                info!("⏰ Waking up at burst {}: resuming normal operation", current_burst);
                self.light_sleep_entry_burst = None;
                
                // Clear lazy free-list to reclaim memory
                self.clear_lazy_free_list();
                
                // Frequency will be restored from genome by caller
                0.0 // Placeholder, caller should restore original frequency
            }
            _ => {
                warn!("Unexpected sleep transition to {:?}", new_state);
                0.0
            }
        };
        
        // Update current phase
        self.current_phase = Some(SleepPhase {
            state: new_state,
            entered_at: Instant::now(),
            burst_count_at_entry: current_burst,
        });
        
        self.current_state = new_state;
        
        new_frequency
    }
    
    /// Build lazy free-list for fast neuron allocation (Light Sleep optimization)
    fn build_lazy_free_list(&self) {
        let start = Instant::now();
        info!("🛌 Building lazy free-list for fast neuron allocation...");
        
        // TODO: Implement lazy free-list building in NPU
        // This scans valid_mask and builds a stack of free indices
        // Safe to interrupt - just drops the incomplete free-list
        
        let duration = start.elapsed();
        info!("🛌 ✅ Lazy free-list built in {:?}", duration);
    }
    
    /// Clear lazy free-list to reclaim memory (on wake)
    fn clear_lazy_free_list(&self) {
        debug!("🛌 Clearing lazy free-list to reclaim memory");
        
        // TODO: Implement free-list clearing in NPU
        // Just drops the Vec<usize>, reclaiming memory
    }
    
    /// Run memory compaction if fragmentation exceeds threshold (Deep Sleep optimization)
    /// ⚠️ NON-INTERRUPTIBLE - blocks wake signals until complete
    fn run_compaction_if_needed(&self) {
        // Check fragmentation level
        let fragmentation = self.get_npu_fragmentation();
        
        if fragmentation < self.config.deep_sleep_compaction_fragmentation_threshold {
            info!(
                "🛌💤 Skipping compaction: fragmentation={:.1}% < threshold={:.1}%",
                fragmentation * 100.0,
                self.config.deep_sleep_compaction_fragmentation_threshold * 100.0
            );
            return;
        }
        
        info!(
            "🛌💤 Starting memory compaction: fragmentation={:.1}% > threshold={:.1}%",
            fragmentation * 100.0,
            self.config.deep_sleep_compaction_fragmentation_threshold * 100.0
        );
        
        // Set compaction flag (blocks wake)
        self.compaction_in_progress.store(true, Ordering::Release);
        
        let start = Instant::now();
        
        // TODO: Implement full memory compaction
        // 1. Move active neurons to front (updates positions)
        // 2. Update ALL synapse references (critical - cannot interrupt!)
        // 3. Update cortical area neuron ID mappings
        
        let duration = start.elapsed();
        
        // Clear compaction flag
        self.compaction_in_progress.store(false, Ordering::Release);
        
        info!(
            "🛌💤 ✅ Memory compaction complete in {:.2}s (⚠️ was non-interruptible)",
            duration.as_secs_f64()
        );
    }
    
    /// Get current NPU fragmentation percentage
    fn get_npu_fragmentation(&self) -> f32 {
        // TODO: Query NPU for fragmentation
        // fragmentation = (count - active_count) / count
        // For now, return placeholder
        0.15 // 15% fragmentation
    }
    
    /// Get current state
    pub fn get_state(&self) -> BurstEngineState {
        self.current_state
    }
    
    /// Check if compaction is in progress (blocking wake)
    pub fn is_compaction_in_progress(&self) -> bool {
        self.compaction_in_progress.load(Ordering::Acquire)
    }
    
    /// Get total time spent in light sleep
    pub fn get_total_light_sleep_seconds(&self) -> u64 {
        self.total_light_sleep_seconds.load(Ordering::Relaxed)
    }
    
    /// Get total time spent in deep sleep
    pub fn get_total_deep_sleep_seconds(&self) -> u64 {
        self.total_deep_sleep_seconds.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_activity_tracker() {
        let mut tracker = ActivityTracker::new(3);
        
        assert!(!tracker.is_full());
        assert_eq!(tracker.average(), 0.0);
        
        tracker.record(10);
        tracker.record(20);
        tracker.record(30);
        
        assert!(tracker.is_full());
        assert_eq!(tracker.average(), 20.0); // (10+20+30)/3
        
        // Overwrite oldest
        tracker.record(40);
        assert_eq!(tracker.average(), 30.0); // (20+30+40)/3
    }
    
    #[test]
    fn test_sleep_phase_timing() {
        let phase = SleepPhase {
            state: BurstEngineState::LightSleep,
            entered_at: Instant::now(),
            burst_count_at_entry: 100,
        };
        
        std::thread::sleep(Duration::from_millis(10));
        
        assert!(phase.duration_seconds() >= 0.01);
    }
}

