use core::fmt::Debug;
use core::hash::Hash;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// All Cortical Area Implementations have their own quantization enums, that must implement this
pub trait CorticalAreaImplementationQuantizationLevelTrait: Clone + Copy + Hash + Eq + PartialEq + Default + Debug {
    /// Calculate the membrane potential level from the given cortical model quantization level. Note
    /// that we do not expect that this be directly encoded in the byte, and should be calculated.
    /// This is alright since this is not used in extremely performance sensitive use cases.
    fn get_membrane_potential_level(&self) -> DecimalQuantizationLevel;
}


#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum CorticalAreaImplementationQuantizationLevel {
    
}