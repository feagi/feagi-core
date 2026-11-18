

//region Indexing Type


/// Defines the index of a given burst engine
#[derive(Clone, Copy, Hash, Debug)]
pub struct BurstEngineIndex(pub u8);

impl core::fmt::Display for BurstEngineIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


/// Defines if engines should be indexed
pub trait EngineIndexingType:
Clone + Copy + core::hash::Hash + core::fmt::Debug + Send + Sync {
    fn into_u8(self) -> u8;
}


/// Effectively no indexing since we have only 1 engine anyways, index will always be 0
#[derive(Clone, Copy, Hash, Debug)]
pub struct SingleEngineIndexing;

impl EngineIndexingType for SingleEngineIndexing {
    fn into_u8(self) -> u8 { 0 } // always 0
}


/// Allows indexing up to 256 burst engines
pub type MultiEngineIndexing = BurstEngineIndex;

impl EngineIndexingType for MultiEngineIndexing {
    fn into_u8(self) -> u8 { self.0 }
}


//endregion