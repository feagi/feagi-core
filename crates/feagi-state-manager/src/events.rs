// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Event streaming for state changes

/// State change event
#[derive(Debug, Clone)]
pub enum StateEvent {
    BurstEngineStateChanged,
    GenomeStateChanged,
    AgentRegistered(String),
    AgentDeregistered(String),
}

/// Event channel
#[cfg(all(feature = "std", not(target_family = "wasm")))]
pub type EventChannel = crossbeam::channel::Sender<StateEvent>;

#[cfg(target_family = "wasm")]
pub struct EventChannel {
    // TODO: Simple buffer for WASM
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
pub struct EventChannel {
    // TODO: heapless::spsc implementation
    _phantom: std::marker::PhantomData<()>,
}
