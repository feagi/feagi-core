//! Neurons in a cortical area may be laid out in some pattern for various use cases. These
//! structs and enums have different ways of describing that.

use core::hash::Hash;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron::model_generated::cortical_layout::{CorticalAreaLayoutNested, CorticalAreaLayoutTypePacked};

/// Represents what type of cortical area layout is being used in a cortical area, within 2 bits
/// (limiting to only 4 options). Describes how the neurons of a cortical area are
/// laid out and any other specific cortical area parameters for that layout
pub trait CorticalAreaLayout<FIQ: FeagiIndexQuantization>: Clone + Hash + PartialEq + Eq {
    /// Returns self as a `PackedCorticalAreaLayoutType`, mainly for use in NPU
    fn to_packed_type(&self) -> CorticalAreaLayoutTypePacked;

    /// As a `CorticalAreaLayoutNested` that also contains the data 
    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ>;

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>;
}


/// Represents the layout of a cortical area with xyz and d dimensions
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutDimensional<FIQ: FeagiIndexQuantization> {
    pub dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutDimensional<FIQ> {
    fn to_packed_type(&self) -> CorticalAreaLayoutTypePacked {
        CorticalAreaLayoutTypePacked::Dimensional
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Dimensional(self)
    }

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant> {
        self.dimensions.number_contained_elements()
    }
}

/// Represents the layout of a cortical with no clear layout (IE memory). A neuron model targeting
/// this cortical layout can use this layout but ALSO any other layout as it does not care about
/// layout anyways so doesn't need it as a parameter.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CorticalAreaLayoutFormless<FIQ: FeagiIndexQuantization> {
    pub neuron_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaLayout<FIQ> for CorticalAreaLayoutFormless<FIQ> {
    fn to_packed_type(&self) -> CorticalAreaLayoutTypePacked {
        CorticalAreaLayoutTypePacked::Formless
    }

    fn get_total_number_neurons(&self) -> NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn to_nested(self) -> CorticalAreaLayoutNested<FIQ> {
        CorticalAreaLayoutNested::Formless(self)
    }
}
