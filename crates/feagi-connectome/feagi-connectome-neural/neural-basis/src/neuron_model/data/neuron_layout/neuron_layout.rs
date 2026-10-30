use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use core::fmt::Debug;
use feagi_basis::collections::spatial_helpers::axis_order::AxisOrder;
use feagi_basis::neurons::{
    CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use feagi_basis::collections::spatial_helpers::spatial_index_context::SpatialHelper;

/// Root trait for defining Neuron Layout, the struct that defines how the neurons are laid out
/// in a cortical area
pub trait NeuronLayout<FIQ: FeagiIndexQuantization>: Clone + Copy + Debug + PartialEq + Sync + Send
+ serde::Serialize + serde::de::DeserializeOwned + Sized + 'static {
    /// What data describes the cortical context that summerizes the state of the whole area
    type CorticalContext;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type PerNeuronContext;

    // TODO layout restrictor

    /// Init the cortical context
    fn new(init_data: Self::CorticalContext) -> Self;

    /// Gets the (max) number of neurons that this layout encloses
    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;

    /// Gets the struct denoting the "location" of a neuron in the context of the cortical area
    fn get_neuron_layout_context(
        &self,
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> Self::PerNeuronContext;

    /// Update the cortical level context. Some implementations have to do some additional things.
    ///
    fn update_cortical_context(&mut self, new_cortical_context: Self::CorticalContext);
}

// TODO Visualization types, depth types, other sub types
