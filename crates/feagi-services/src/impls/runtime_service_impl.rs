/*!
Runtime control service implementation.

Provides control over the FEAGI burst engine runtime.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use async_trait::async_trait;
use feagi_burst_engine::BurstLoopRunner;
use std::sync::{Arc, Mutex};

use crate::traits::RuntimeService;
use crate::types::{RuntimeStatus, ServiceError, ServiceResult};

/// Default implementation of RuntimeService
///
/// Wraps the BurstLoopRunner and provides async interface for runtime control.
pub struct RuntimeServiceImpl {
    burst_runner: Arc<Mutex<BurstLoopRunner>>,
    paused: Arc<parking_lot::RwLock<bool>>,
}

impl RuntimeServiceImpl {
    /// Create a new RuntimeServiceImpl
    pub fn new(burst_runner: Arc<Mutex<BurstLoopRunner>>) -> Self {
        Self {
            burst_runner,
            paused: Arc::new(parking_lot::RwLock::new(false)),
        }
    }
}

#[async_trait]
impl RuntimeService for RuntimeServiceImpl {
    async fn start(&self) -> ServiceResult<()> {
        log::info!("Starting burst engine");
        
        let mut runner = self.burst_runner.lock().unwrap();
        
        runner
            .start()
            .map_err(|e| ServiceError::InvalidState(e.to_string()))?;
        
        // Clear paused flag
        *self.paused.write() = false;
        
        Ok(())
    }

    async fn stop(&self) -> ServiceResult<()> {
        log::info!("Stopping burst engine");
        
        let mut runner = self.burst_runner.lock().unwrap();
        runner.stop();
        
        // Clear paused flag
        *self.paused.write() = false;
        
        Ok(())
    }

    async fn pause(&self) -> ServiceResult<()> {
        log::info!("Pausing burst engine");
        
        let runner = self.burst_runner.lock().unwrap();
        if !runner.is_running() {
            return Err(ServiceError::InvalidState(
                "Burst engine is not running".to_string(),
            ));
        }
        
        // Set paused flag (actual pause implementation depends on burst loop design)
        *self.paused.write() = true;
        
        // TODO: Implement actual pause mechanism in BurstLoopRunner
        // For now, we just track the paused state
        log::warn!("Pause not yet implemented in BurstLoopRunner - using flag only");
        
        Ok(())
    }

    async fn resume(&self) -> ServiceResult<()> {
        log::info!("Resuming burst engine");
        
        let paused = *self.paused.read();
        if !paused {
            return Err(ServiceError::InvalidState(
                "Burst engine is not paused".to_string(),
            ));
        }
        
        // Clear paused flag
        *self.paused.write() = false;
        
        // TODO: Implement actual resume mechanism in BurstLoopRunner
        log::warn!("Resume not yet implemented in BurstLoopRunner - using flag only");
        
        Ok(())
    }

    async fn step(&self) -> ServiceResult<()> {
        log::info!("Executing single burst step");
        
        let runner = self.burst_runner.lock().unwrap();
        if runner.is_running() {
            return Err(ServiceError::InvalidState(
                "Cannot step while burst engine is running in continuous mode".to_string(),
            ));
        }
        
        // TODO: Implement single-step execution in BurstLoopRunner
        log::warn!("Single-step execution not yet implemented in BurstLoopRunner");
        
        Err(ServiceError::NotImplemented(
            "Single-step execution not yet implemented".to_string(),
        ))
    }

    async fn get_status(&self) -> ServiceResult<RuntimeStatus> {
        let runner = self.burst_runner.lock().unwrap();
        let is_running = runner.is_running();
        let burst_count = runner.get_burst_count();
        let is_paused = *self.paused.read();
        
        // Note: Some metrics not yet available from BurstLoopRunner
        // - current_rate_hz: Would require tracking actual execution rate
        // - last_burst_neuron_count: Not tracked by BurstLoopRunner
        // - avg_burst_time_ms: Not tracked by BurstLoopRunner
        Ok(RuntimeStatus {
            is_running,
            is_paused,
            frequency_hz: runner.get_frequency(),
            burst_count,
            current_rate_hz: if is_running { runner.get_frequency() } else { 0.0 },
            last_burst_neuron_count: 0, // Not yet tracked
            avg_burst_time_ms: 0.0,     // Not yet tracked
        })
    }

    async fn set_frequency(&self, frequency_hz: f64) -> ServiceResult<()> {
        if frequency_hz <= 0.0 {
            return Err(ServiceError::InvalidInput(
                "Frequency must be greater than 0".to_string(),
            ));
        }
        
        log::info!("Setting burst frequency to {} Hz", frequency_hz);
        
        let mut runner = self.burst_runner.lock().unwrap();
        runner.set_frequency(frequency_hz);
        
        Ok(())
    }

    async fn get_burst_count(&self) -> ServiceResult<u64> {
        let runner = self.burst_runner.lock().unwrap();
        Ok(runner.get_burst_count())
    }

    async fn reset_burst_count(&self) -> ServiceResult<()> {
        log::info!("Resetting burst count");
        
        // TODO: Implement burst count reset in BurstLoopRunner
        log::warn!("Burst count reset not yet implemented in BurstLoopRunner");
        
        Err(ServiceError::NotImplemented(
            "Burst count reset not yet implemented".to_string(),
        ))
    }
}

