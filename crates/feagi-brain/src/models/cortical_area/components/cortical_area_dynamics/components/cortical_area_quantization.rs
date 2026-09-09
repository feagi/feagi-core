use std::hash::Hash;
use std::marker::PhantomData;
use feagi_data::feagi_data_neuron::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::feagi_data_quantization::prelude::QuantizedDecimalUnwrappedTrait;
use feagi_data::feagi_data_quantization::values::quantizable::DecimalQuantizationLevel;

/// Common root trait shared by all Neuron Model Quantizations. This trait should be extended
/// by the given neuron model to add any quantization parameters for their given data
pub trait CorticalAreaQuantization: MembranePotentialQuantization {
    
    // /// All quantizations used by a given cortical model quantization level. Useful for validating
    // /// device compatibility. This will also be extended in extensions of this trait
    // const USED_DECIMAL_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel]; // Don't include a default, as we then forget about it
}

// Putting this here as this is the only "generic" concrete implementation. 
// Most cortical models have their specific CorticalAreaQuantization implementation for themselves.
//region Null Implementation

/// Used by any cortical model that has no unique data pertaining to itself, 
/// other than membrane potential which is universally required
#[derive(Clone, Copy, Debug)]
pub struct NullCorticalAreaQuantization<MembranePotential: QuantizedDecimalUnwrappedTrait>(PhantomData<MembranePotential>);


impl<MembranePotential: QuantizedDecimalUnwrappedTrait> MembranePotentialQuantization for NullCorticalAreaQuantization<MembranePotential> {
    type MembranePotentialQuant = MembranePotential;
}

impl<MembranePotential: QuantizedDecimalUnwrappedTrait> CorticalAreaQuantization for NullCorticalAreaQuantization<MembranePotential> {
    // nothing!
}

impl<MembranePotential: QuantizedDecimalUnwrappedTrait> NullCorticalAreaQuantization<MembranePotential>
{
    pub fn new() -> NullCorticalAreaQuantization<MembranePotential> {
        NullCorticalAreaQuantization(PhantomData)
    }
}

//endregion
