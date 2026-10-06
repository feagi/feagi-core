
/// Can be used to label a specific instance of an item within some BrainGraph
pub trait BrainGraphIdentifier: core::fmt::Debug + core::fmt::Display +
Clone + Copy + core::hash::Hash + core::str::FromStr + Sized
+ serde::Serialize + serde::de::DeserializeOwned + 'static
{}

/// Specifies a specific instance of an item in a genome (actually exists)
pub trait GenomeIdentifier: BrainGraphIdentifier {}

/// Specifies an instance of some item that is being imported / exported within relation to that
/// grouping, not the greater graph. intended to be replaced
pub trait ImposterIdentifier: BrainGraphIdentifier {
    // https://www.youtube.com/watch?v=ZR18zjpu0Hk
    /// What this is the imposter of
    type Impostering: GenomeIdentifier;
}