use feagi_data::feagi_data_error::{FeagiDataError, FeagiFailDataEtc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

fn feagi_data_etc_error(message: String) -> FeagiDataError {
    let context: &'static str = Box::leak(message.into_boxed_str());
    FeagiFailDataEtc::new(context).into()
}

/// A unique identifier for a subscription to a [`FeagiSignal`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FeagiSignalIndex(u32);

impl FeagiSignalIndex {
    pub const fn from(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl std::ops::Deref for FeagiSignalIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for FeagiSignalIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<FeagiSignalIndex> for u32 {
    fn from(value: FeagiSignalIndex) -> Self {
        value.0
    }
}

impl std::fmt::Display for FeagiSignalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type SignalListener<T> = Box<dyn FnMut(&T) + Send>;

/// Event signal system similar to Godot signals.
pub struct FeagiSignal<T> {
    listeners: HashMap<FeagiSignalIndex, SignalListener<T>>,
    next_index: u32,
}

impl<T> FeagiSignal<T> {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn connect<F>(&mut self, f: F) -> FeagiSignalIndex
    where
        F: FnMut(&T) + Send + 'static,
    {
        let index = FeagiSignalIndex::from(self.next_index);
        self.listeners.insert(index, Box::new(f));
        self.next_index += 1;
        index
    }

    pub fn disconnect(&mut self, index: FeagiSignalIndex) -> Result<(), FeagiDataError> {
        if self.listeners.remove(&index).is_some() {
            return Ok(());
        }
        Err(feagi_data_etc_error(format!(
            "No subscription found with identifier {}!",
            index
        )))
    }

    pub fn emit(&mut self, value: &T) {
        for f in self.listeners.values_mut() {
            f(value);
        }
    }

    pub fn connect_with_shared_state<S, F>(
        &mut self,
        state: Arc<Mutex<S>>,
        mut callback: F,
    ) -> FeagiSignalIndex
    where
        S: Send + 'static,
        F: FnMut(&mut S, &T) + Send + 'static,
    {
        self.connect(move |event| {
            if let Ok(mut guard) = state.lock() {
                callback(&mut *guard, event);
            }
        })
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    pub fn disconnect_all(&mut self) {
        self.listeners.clear();
    }
}

impl<T> Default for FeagiSignal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for FeagiSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeagiSignal")
            .field("listener_count", &self.listeners.len())
            .field("next_index", &self.next_index)
            .field(
                "listener_indices",
                &self.listeners.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}
