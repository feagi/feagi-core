// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Async burst loop implementation using FeagiAsyncRuntime.

This module provides an async version of the burst loop that can run
on both desktop (Tokio) and WASM (wasm-bindgen-futures) platforms.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::DynamicNPU;
use crate::burst_loop_runner::{VisualizationPublisher, MotorPublisher, RawFireQueueSnapshot};
use feagi_async::FeagiAsyncRuntime;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{info, warn};
use parking_lot::RwLock as ParkingLotRwLock;
use ahash::AHashMap;
use crate::parameter_update_queue::ParameterUpdateQueue;

/// Async burst loop that uses FeagiAsyncRuntime for platform-agnostic async operations
///
/// This is the async equivalent of the synchronous `burst_loop` function.
/// It can run on both desktop (Tokio) and WASM platforms.
pub async fn async_burst_loop<R: FeagiAsyncRuntime>(
    runtime: Arc<R>,
    npu: Arc<Mutex<DynamicNPU>>,
    frequency_hz: Arc<Mutex<f64>>,
    running: Arc<AtomicBool>,
    viz_publisher: Option<Arc<dyn VisualizationPublisher>>,
    motor_publisher: Option<Arc<dyn MotorPublisher>>,
    motor_subscriptions: Arc<ParkingLotRwLock<AHashMap<String, ahash::AHashSet<String>>>>,
    cached_burst_count: Arc<std::sync::atomic::AtomicU64>,
    parameter_queue: ParameterUpdateQueue,
) {
    info!("[ASYNC-BURST-LOOP] Starting async burst loop");
    
    let mut burst_num = 0u64;
    
    while running.load(Ordering::Acquire) {
        let burst_start = std::time::Instant::now();
        
        // Process burst
        let burst_result = {
            let mut npu_guard = npu.lock().unwrap();
            npu_guard.process_burst()
        };
        
        match burst_result {
            Ok(result) => {
                burst_num += 1;
                cached_burst_count.store(burst_num, Ordering::Relaxed);
                
                // TODO: Handle visualization and motor publishing
                // This will be implemented in Phase 2
                
                // Calculate delay for next burst
                let frequency = *frequency_hz.lock().unwrap();
                let interval_sec = 1.0 / frequency;
                let burst_duration = burst_start.elapsed();
                let sleep_duration = if interval_sec > burst_duration.as_secs_f64() {
                    Duration::from_secs_f64(interval_sec - burst_duration.as_secs_f64())
                } else {
                    Duration::ZERO
                };
                
                // Use async delay instead of thread::sleep
                if sleep_duration > Duration::ZERO {
                    runtime.delay(sleep_duration).await;
                }
            }
            Err(e) => {
                warn!("[ASYNC-BURST-LOOP] Burst processing error: {}", e);
                // Continue loop even on error
                runtime.delay(Duration::from_millis(10)).await;
            }
        }
        
        // Check shutdown flag
        if !running.load(Ordering::Relaxed) {
            break;
        }
    }
    
    info!("[ASYNC-BURST-LOOP] Burst loop stopped");
}

