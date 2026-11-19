use crate::genome::editing::steps::GenomeEditStepEnum;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub mod load_genome;
pub mod amalgamate_genome;
pub mod edit_genome;
pub mod embodiment_joining;

/// A serializable ordered list of requests to edit the Genome (and Connectome)
pub trait GenomeEditRequests<FIQ>: Clone + Debug + Serialize + DeserializeOwned
where 
    FIQ: FeagiIndexQuantization,
{
    /// The type of enum holding the steps. Different enums restrict to different possible steps 
    type EditStepEnum: GenomeEditStepEnum<FIQ>;
    
    /// Get what intensity the current edit sequence would be (assuming validity)
    fn get_intensity(&self) -> GenomeEditSequenceIntensity;
    
    /// Add a single step to the end of the list of requests of edits
    fn add_step(&mut self, step: impl Into<Self::EditStepEnum>);

    /// Iterate over the steps of this request, in the order they will be applied
    fn iter_steps(&self) -> impl Iterator<Item = &Self::EditStepEnum>;

    
    
}


//endregion


#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenomeEditSequenceIntensity {
    /// No edits defined, will be rejected
    Empty = 0,
    /// Edit only touches the genome, no connectome
    GenomeOnly = 1,
    /// Edit involves the connectome but (most likely) does not involve
    /// allocation / deallocation, which is slow
    ConnectomeInPlace = 2,
    /// Requires the connectome to expand / contract, which would be slow
    ConnectomeAllocating = 3,
    /// (Re)laods a genome, guaranteed interruption
    ConnectomeReloading = 4,
}