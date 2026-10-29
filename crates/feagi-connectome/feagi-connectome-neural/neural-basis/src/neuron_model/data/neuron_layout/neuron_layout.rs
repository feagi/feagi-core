use core::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use feagi_basis::collections::spatial_helpers::axis_order::AxisOrder;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root trait for defining Neuron Layout. Implemented on a marker type that holds no data, with
/// the data of a given layout at the runtimes index quantization being [`NeuronLayout::Data`]
pub trait NeuronLayout: Clone + Copy + Debug + PartialEq + Sync + Send + 'static
{
    /// The per instance data of the layout (length, dimensions, etc)
    type Data<FIQ: FeagiIndexQuantization>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync;

    /// What data describes the cortical context for a neuron, that has a method that given
    /// the neuron local index, can return the
    type CorticalContext<FIQ: FeagiIndexQuantization>: Clone + Serialize + Sized;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type PerNeuronContext<FIQ: FeagiIndexQuantization>: Clone + Serialize + Sized;

    fn build_new<FIQ: FeagiIndexQuantization>(cortical_context: Self::CorticalContext<FIQ>) -> Self::Data<FIQ>;

    /// Gets the (max) number of neurons that this layout encloses
    fn get_neuron_count<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;

    /// Gets the struct ref governing the total layout of all the neurons
    fn get_cortical_layout_context<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> &Self::CorticalContext<FIQ>;

    /// Gets the struct denoting the "location" of a neuron in the context of the cortical area
    fn get_neuron_layout_context<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext<FIQ>;

    fn update_cortical_context<FIQ: FeagiIndexQuantization>(data: &mut Self::Data<FIQ>, cortical_context: Self::CorticalContext<FIQ>);
}


// TODO Visualization types, depth types, other sub types

/// Defines some pattern of laying out voxels in a 3D voxel grid (with density in the fourth dim)
pub trait VoxelNeuronLayout: NeuronLayout
{
    /// The order the axes are laid out in
    type AxisOrder: AxisOrder<4>;

    /// True if the axis order is the default incrementing order
    const IS_AXIS_ORDER_DEFAULT: bool;

    /// True if more than 1 neuron is allowed per voxels
    const VOXELS_ALLOWED_DENSE: bool;

    /// Gets the voxel dimensions of the layout
    fn get_dimensions<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;

    /// Gets the voxel coordinate (with depth) of a neuron
    fn linear_to_coordinate<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant>;
}
