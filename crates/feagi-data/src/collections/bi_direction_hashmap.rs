use ahash::AHashMap;
use std::hash::Hash;

// TODO We can probably optimize this a bit

/// A simple Hashmap implementation with forward and backward searching.
pub struct BiDirectionHashmap<ForKey, BackKey>
where
    ForKey: Eq + Hash + Clone,
    BackKey: Eq + Hash + Clone,
{
    forward_hash: AHashMap<ForKey, BackKey>,
    backward_hash: AHashMap<BackKey, ForKey>,
}

impl<ForKey, BackKey> BiDirectionHashmap<ForKey, BackKey>
where
    ForKey: Eq + Hash + Clone,
    BackKey: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            forward_hash: Default::default(),
            backward_hash: Default::default(),
        }
    }

    pub fn get_forward(&self, key: &ForKey) -> Option<&BackKey> {
        self.forward_hash.get(key)
    }

    pub fn get_back(&self, key: &BackKey) -> Option<&ForKey> {
        self.backward_hash.get(key)
    }

    pub fn get_forward_mut(&mut self, key: &ForKey) -> Option<&mut BackKey> {
        self.forward_hash.get_mut(key)
    }

    pub fn get_backward_mut(&mut self, key: &BackKey) -> Option<&mut ForKey> {
        self.backward_hash.get_mut(key)
    }

    pub fn remove_forward(&mut self, forward_key: &ForKey) -> Option<()> {
        let back = self.forward_hash.remove(forward_key)?;
        self.backward_hash.remove(&back)?;
        Some(())
    }

    pub fn remove_backward(&mut self, backward_key: &BackKey) -> Option<()> {
        let forward = self.backward_hash.remove(backward_key)?;
        self.forward_hash.remove(&forward)?;
        Some(())
    }

    pub fn insert(&mut self, forward_key: ForKey, backward_key: BackKey) {
        self.forward_hash.insert(forward_key.clone(), backward_key.clone());
        self.backward_hash.insert(backward_key, forward_key);
    }
}
