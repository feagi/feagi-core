use serde::{Deserialize, Serialize};
use feagi_basis::collections::spatial_helpers::axis_order::AxisOrder;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root trait for defining Neuron Layout
pub trait NeuronLayout<FIQ: FeagiIndexQuantization>: Clone + Serialize + Sized
{
    /// What data describes the cortical context for a neuron, that has a method that given
    /// the neuron local index, can return the
    type CorticalContext: Clone + Serialize + Sized;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type PerNeuronContext: Clone + Serialize + Sized;

    fn build_new(cortical_context: Self::CorticalContext) -> Self;
    
    /// Gets the (max) number of neurons that this layout encloses
    fn get_neuron_count(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;

    /// Gets the struct ref governing the total layout of all the neurons
    fn get_cortical_layout_context(&self) -> &Self::CorticalContext;

    /// Gets the struct denoting the "location" of a neuron in the context of the cortical area
    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext;

    fn update_cortical_context(&mut self, cortical_context: Self::CorticalContext);
}


// TODO Visualization types, depth types, other sub types

/// Defines some pattern of laying out voxels in a 3D voxel grid (with density in the fourth dim)
pub trait VoxelNeuronLayout<FIQ, AxisOrderType>:
NeuronLayout<
    FIQ,
    CorticalContext=DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    PerNeuronContext=DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>,
>
where
    FIQ: FeagiIndexQuantization,
    AxisOrderType: AxisOrder<4>
{
    /// True if the axis order is the default incrementing order
    const IS_AXIS_ORDER_DEFAULT: bool;

    /// True if more than 1 neuron is allowed per voxels
    const VOXELS_ALLOWED_DENSE: bool;
}