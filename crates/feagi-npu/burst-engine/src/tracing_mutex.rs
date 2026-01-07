// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Tracing Mutex wrapper that automatically logs all lock acquisitions and releases
//! This provides visibility into lock contention across all code paths
//! 
//! When `npu-lock-tracing` feature is disabled, this module provides a zero-overhead
//! wrapper that behaves identically to std::sync::Mutex

use std::sync::Mutex;
use std::time::Instant;
use std::thread;

#[cfg(feature = "npu-lock-tracing")]
use tracing::{debug, warn};

/// A wrapper around `Mutex<T>` that conditionally logs all lock operations
/// When `npu-lock-tracing` feature is enabled, logs all lock acquisitions/releases
/// When disabled, zero-overhead wrapper identical to std::sync::Mutex
pub struct TracingMutex<T> {
    inner: Mutex<T>,
    #[cfg(feature = "npu-lock-tracing")]
    name: &'static str,
}

impl<T> TracingMutex<T> {
    /// Create a new TracingMutex with the given value and name
    /// Name parameter is only used when `npu-lock-tracing` feature is enabled
    pub fn new(value: T, _name: &'static str) -> Self {
        Self {
            inner: Mutex::new(value),
            #[cfg(feature = "npu-lock-tracing")]
            name: _name,
        }
    }

    /// Acquire the lock, conditionally logging the acquisition and release
    #[cfg(feature = "npu-lock-tracing")]
    pub fn lock(&self) -> Result<TracingMutexGuard<'_, T>, std::sync::PoisonError<MutexGuard<'_, T>>> {
        let thread_id = thread::current().id();
        let lock_start = Instant::now();

        match self.inner.lock() {
            Ok(guard) => {
                let lock_acquired = Instant::now();
                let lock_wait = lock_acquired.duration_since(lock_start);
                
                // Only log at warn level if wait time is significant (> 5ms)
                // Fast acquisitions are normal (especially for burst loop) - use debug level
                if lock_wait.as_millis() > 5 {
                    warn!(
                        "[NPU-LOCK-TRACE] {}: Thread {:?} acquired lock after {:.2}ms wait (SLOW)",
                        self.name,
                        thread_id,
                        lock_wait.as_secs_f64() * 1000.0
                    );
                } else {
                    debug!(
                        "[NPU-LOCK-TRACE] {}: Thread {:?} acquired lock immediately ({:.2}ms wait)",
                        self.name,
                        thread_id,
                        lock_wait.as_secs_f64() * 1000.0
                    );
                }

                Ok(TracingMutexGuard {
                    guard,
                    name: self.name,
                    thread_id,
                    acquire_time: lock_acquired,
                    wait_duration: lock_wait,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Acquire the lock (zero-overhead when tracing disabled)
    #[cfg(not(feature = "npu-lock-tracing"))]
    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, T>, std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> {
        self.inner.lock()
    }

    /// Try to acquire the lock without blocking (with tracing if enabled)
    #[cfg(feature = "npu-lock-tracing")]
    pub fn try_lock(&self) -> Result<TracingMutexGuard<'_, T>, std::sync::TryLockError<MutexGuard<'_, T>>> {
        let thread_id = thread::current().id();
        
        match self.inner.try_lock() {
            Ok(guard) => {
                let lock_acquired = Instant::now();
                // try_lock is used by non-burst-loop threads (e.g., ConnectomeManager cache updates)
                // Always log at warn level to track non-burst-loop NPU access
                warn!(
                    "[NPU-LOCK-TRACE] {}: Thread {:?} acquired lock via try_lock (NON-BURST-LOOP) - operation starting",
                    self.name,
                    thread_id
                );
                Ok(TracingMutexGuard {
                    guard,
                    name: self.name,
                    thread_id,
                    acquire_time: lock_acquired,
                    wait_duration: std::time::Duration::ZERO,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Try to acquire the lock without blocking (zero-overhead when tracing disabled)
    #[cfg(not(feature = "npu-lock-tracing"))]
    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<'_, T>, std::sync::TryLockError<std::sync::MutexGuard<'_, T>>> {
        self.inner.try_lock()
    }

    /// Get a reference to the inner Mutex (for advanced use cases)
    pub fn inner(&self) -> &Mutex<T> {
        &self.inner
    }

    /// Convert TracingMutex to inner Mutex (for compatibility with code that hasn't been updated yet)
    /// WARNING: This bypasses tracing for lock acquisitions on the returned Mutex
    pub fn into_inner(self) -> Mutex<T> {
        self.inner
    }
}

/// Guard returned by TracingMutex::lock() that logs when it's dropped (only when tracing enabled)
#[cfg(feature = "npu-lock-tracing")]
pub struct TracingMutexGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    name: &'static str,
    thread_id: thread::ThreadId,
    acquire_time: Instant,
    wait_duration: std::time::Duration,
}

#[cfg(feature = "npu-lock-tracing")]
impl<'a, T> std::ops::Deref for TracingMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

#[cfg(feature = "npu-lock-tracing")]
impl<'a, T> std::ops::DerefMut for TracingMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

#[cfg(feature = "npu-lock-tracing")]
impl<'a, T> Drop for TracingMutexGuard<'a, T> {
    fn drop(&mut self) {
        let release_time = Instant::now();
        let hold_duration = release_time.duration_since(self.acquire_time);
        let total_duration = hold_duration + self.wait_duration;

        // Log release based on hold time and wait time
        // Long holds (> 5ms) or long waits (> 5ms) are suspicious - log at warn level
        let was_slow = hold_duration.as_millis() > 5 || self.wait_duration.as_millis() > 5;
        
        if was_slow {
            warn!(
                "[NPU-LOCK-TRACE] {}: Thread {:?} RELEASED lock (held for {:.2}ms, total from attempt: {:.2}ms)",
                self.name,
                self.thread_id,
                hold_duration.as_secs_f64() * 1000.0,
                total_duration.as_secs_f64() * 1000.0
            );
        } else {
            debug!(
                "[NPU-LOCK-TRACE] {}: Thread {:?} RELEASED lock (held for {:.2}ms)",
                self.name,
                self.thread_id,
                hold_duration.as_secs_f64() * 1000.0
            );
        }
    }
}

