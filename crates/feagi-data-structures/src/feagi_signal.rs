use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use crate::{define_index, FeagiDataError};

define_index!(FeagiSignalIndex, u32, "A unique identifier for a subscription to a FeagiSignal");

/// Event signal system similar to Godot signals.
/// 
/// Allows subscribing callbacks that will be invoked when events are emitted.
/// Uses `FnMut` to allow closures to capture and modify external state via `Arc<Mutex<T>>`.
/// 
/// # Example with Shared State
/// ```
/// use std::sync::{Arc, Mutex};
/// use feagi_data_structures::FeagiSignal;
/// 
/// struct MyHandler {
///     count: i32,
/// }
/// 
/// impl MyHandler {
///     fn handle_event(&mut self, data: &String) {
///         self.count += 1;
///         println!("Event {}: {}", self.count, data);
///     }
/// }
/// 
/// let handler = Arc::new(Mutex::new(MyHandler { count: 0 }));
/// let mut signal = FeagiSignal::new();
/// 
/// // Clone Arc for the closure
/// let handler_clone = Arc::clone(&handler);
/// signal.connect(move |data| {
///     handler_clone.lock().unwrap().handle_event(data);
/// });
/// 
/// signal.emit(&"Hello".to_string());
/// assert_eq!(handler.lock().unwrap().count, 1);
/// ```
pub struct FeagiSignal<T> {
    listeners: HashMap<FeagiSignalIndex, Box<dyn FnMut(&T) + Send>>,
    next_index: u32,
}


impl<T> FeagiSignal<T> {
    /// Creates a new empty signal.
    pub fn new() -> Self {
        Self { listeners: HashMap::new(), next_index: 0 }
    }

    /// Connects a callback to this signal.
    /// 
    /// The callback will be invoked whenever `emit()` is called.
    /// Returns a handle that can be used to disconnect later.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::FeagiSignal;
    /// 
    /// let mut signal = FeagiSignal::new();
    /// let handle = signal.connect(|value: &i32| {
    ///     println!("Received: {}", value);
    /// });
    /// 
    /// signal.emit(&42);
    /// signal.disconnect(handle).unwrap();
    /// ```
    pub fn connect<F>(&mut self, f: F) -> FeagiSignalIndex // Will overflow after 4 billion subscriptions. Too bad!
    where
        F: FnMut(&T) + Send + 'static,
    {
        self.listeners.insert(self.next_index.into(), Box::new(f));
        self.next_index += 1;
        (self.next_index - 1).into()
    }

    /// Disconnects a previously connected callback.
    /// 
    /// Returns an error if no callback with the given index exists.
    pub fn disconnect(&mut self, index: FeagiSignalIndex) -> Result<(), FeagiDataError> {
        if self.listeners.remove(&index).is_some() {
            return Ok(())
        }
        Err(FeagiDataError::BadParameters(format!("No subscription found with identifier {}!", index)))
    }

    /// Emits an event to all connected callbacks.
    /// 
    /// Callbacks are invoked in arbitrary order.
    pub fn emit(&mut self, value: &T) {
        for f in self.listeners.values_mut() {
            f(value);
        }
    }
    
    /// Helper to connect a closure that captures an `Arc<Mutex<S>>`.
    /// 
    /// This is a convenience method for the common pattern of calling methods
    /// on shared mutable state from within the callback.
    /// 
    /// # Example
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use feagi_data_structures::FeagiSignal;
    /// 
    /// struct Counter { value: i32 }
    /// impl Counter {
    ///     fn increment(&mut self) { self.value += 1; }
    /// }
    /// 
    /// let counter = Arc::new(Mutex::new(Counter { value: 0 }));
    /// let mut signal = FeagiSignal::new();
    /// 
    /// signal.connect_with_shared_state(
    ///     Arc::clone(&counter),
    ///     |state, _event| state.increment()
    /// );
    /// 
    /// signal.emit(&"event");
    /// assert_eq!(counter.lock().unwrap().value, 1);
    /// ```
    pub fn connect_with_shared_state<S, F>(&mut self, state: Arc<Mutex<S>>, mut callback: F) -> FeagiSignalIndex
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
    
    /// Returns the number of connected listeners.
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
    
    /// Removes all connected listeners.
    pub fn disconnect_all(&mut self) {
        self.listeners.clear();
    }
}

// Note: For PyO3 users, you can use this pattern:
//
// ```rust,ignore
// use pyo3::prelude::*;
// use pyo3::types::PyAny;
// use std::sync::Arc;
// 
// #[pyclass]
// struct MyPythonHandler {
//     callback: Arc<Py<PyAny>>, // Python callable stored as Arc
// }
// 
// impl MyPythonHandler {
//     fn handle_event(&self, py: Python, data: &MyEventType) -> PyResult<()> {
//         self.callback.call1(py, (data.clone(),))?;
//         Ok(())
//     }
// }
// 
// // In your PyO3 setup:
// let handler = Arc::new(MyPythonHandler { callback: Arc::new(python_fn) });
// let handler_clone = Arc::clone(&handler);
// 
// signal.connect(move |event| {
//     Python::with_gil(|py| {
//         let _ = handler_clone.handle_event(py, event);
//     });
// });
// ```

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
            .field("listener_indices", &self.listeners.keys().collect::<Vec<_>>())
            .finish()
    }
}