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
#[cfg(feature = "std")]
pub type EventChannel = crossbeam::channel::Sender<StateEvent>;

#[cfg(feature = "no_std")]
pub struct EventChannel {
    // TODO: heapless::spsc implementation
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(target_family = "wasm")]
pub struct EventChannel {
    // TODO: Simple buffer for WASM
    _phantom: std::marker::PhantomData<()>,
}





