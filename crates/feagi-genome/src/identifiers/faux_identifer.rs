use serde::{Deserialize, Serialize};
use feagi_basis::identifiers::Identifier;


/// The given identifier has a faux identifier
pub trait HasFauxIdentifier: Identifier {}

/// Used when moving identifier structures with relationships between each other but
/// you do not want to risk collision
#[derive(Hash, Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct FauxIdentifier<I: HasFauxIdentifier>(u32, core::marker::PhantomData<I>);

impl<I: HasFauxIdentifier> FauxIdentifier<I> {
    pub fn new() -> Self {
        Self(0, core::marker::PhantomData)
    } 
    
    pub fn to_u32(self) -> u32 {
        self.0
    }
    
    /// Returns make a copy of self to return, and then increment self
    pub fn copy_and_increment(&mut self) -> Self {
        let c = self.clone();
        self.0 += 1;
        c
    }
}

impl<I: HasFauxIdentifier> Identifier for FauxIdentifier<I> {}


impl<I: HasFauxIdentifier> Default for FauxIdentifier<I> {
    fn default() -> Self {
        Self::new()
    }
}
