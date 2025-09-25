use std::collections::HashMap;
use crate::define_index;

define_index!(FeagiSignalIndex, u32, "A unique identifier for a subscription to a FeagiSignal");

pub struct FeagiSignal<T> { // Totally not stolen concept form Godot
    listeners: HashMap<FeagiSignalIndex, Box<dyn Fn(&T) + Send + Sync>>,
    next_index: u32,
}

impl<T> FeagiSignal<T> {
    pub fn new() -> Self {
        Self { listeners: HashMap::new(), next_index: 0 }
    }
    
    pub fn connect<F>(&mut self, f: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.listeners.insert(self.next_index.into(), Box::new(f));
        self.next_index += 1;
    }

    pub fn emit(&self, value: T) {
        for f in &self.listeners {
            f.1(&value);
        }
    }
}