use std::collections::HashMap;
use feagi_data_structures::wrapped_io_data::{WrappedIOType, WrappedIOData};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CallbackSubscriberID {
    pub id: usize
}

impl CallbackSubscriberID {
    pub fn new(id: usize) -> CallbackSubscriberID {
        CallbackSubscriberID { id }
    }
}

pub struct CallBackManager {
    callbacks: HashMap<usize, Box<dyn Fn(&WrappedIOData) + Send + Sync>>,
    next_id: usize, // This will fail if more than 18,446,744,073,709,551,615 callbacks are registered (on 64bit). Too Bad!
    data_type: WrappedIOType,
}

impl  CallBackManager {
    pub fn new(data_type: WrappedIOType) -> CallBackManager {
        Self {
            callbacks: HashMap::new(),
            next_id: 0,
            data_type,
        }
    }

    pub fn register(&mut self, callback:  Box<dyn Fn(&WrappedIOData) + Send + Sync>) -> CallbackSubscriberID
    {
        let id = self.next_id;
        self.callbacks.insert(id, callback);
        self.next_id += 1;
        CallbackSubscriberID::new(id)
    }

    /// Deregister a callback by ID
    pub fn deregister(&mut self, id: CallbackSubscriberID) -> bool {
        self.callbacks.remove(&id.id).is_some()
    }

    /// Call all registered callbacks with a parameter
    pub(crate) fn emit(&self, value: &WrappedIOData) {
        for cb in self.callbacks.values() {
            cb(value);
        }
    }
    
}


