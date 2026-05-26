use std::hash::Hash;
use ahash::AHashMap;
use feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::FeagiCommonError;
use crate::useful_structs::{IndexedDataTracker};

pub struct BiDirectionalHashMap<LookupForward, LookupBackward, InternalIndexType, Data>
where
    LookupForward: Hash,
    LookupBackward: Hash,
    InternalIndexType: QuantizedIndexCountTrait,
{
    index_tracker: IndexedDataTracker<Data, InternalIndexType>,
    forward_lookup: AHashMap<LookupForward, InternalIndexType>,
    backward_lookup: AHashMap<LookupBackward, InternalIndexType>,
}

impl<LookupForward, LookupBackward, InternalIndexType, Data> BiDirectionalHashMap<LookupForward, LookupBackward, InternalIndexType, Data>
where
    LookupForward: Hash,
    LookupBackward: Hash,
    InternalIndexType: QuantizedIndexCountTrait,
{
    
    pub fn new() -> Self {
        Self {
            index_tracker: IndexedDataTracker::new(),
            forward_lookup: AHashMap::new(),
            backward_lookup: AHashMap::new(),
        }
    }
    
    pub fn lookup_forwards(&self, lookup_forward: &LookupForward) -> Option<&Data> {
        let index = self.forward_lookup.get(lookup_forward);
        self.index_tracker.get(index)
    }
    
    pub fn lookup_backwards(&self, lookup_backward: &LookupBackward) -> Option<&Data> {
        let index = self.backward_lookup.get(lookup_backward);
        self.index_tracker.get(index)
    }
    
    pub fn insert_checked(&mut self, lookup_forward: LookupForward, lookup_backward: LookupBackward, data: Data) -> Result<(), FeagiCommonError> {
        
    }

    pub fn remove_checked(&mut self, lookup_forward: LookupForward, lookup_backward: LookupBackward) -> Result<(), FeagiCommonError> {

    }
    
    
    pub fn insert_unchecked(&mut self, lookup_forward: LookupForward, lookup_backward: LookupBackward, data: Data) {
        
    }

    pub fn remove_unchecked(&mut self, lookup_forward: LookupForward, lookup_backward: LookupBackward) {

    }
    
}