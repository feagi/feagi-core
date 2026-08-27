//! Used for "data empty" cortical areas that dont need to hold any state

use core::hash::{Hash};
use core::marker::PhantomData;
use std::fmt::{Debug, Formatter};
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::QuantizedDecimalUnwrappedTrait;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;

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