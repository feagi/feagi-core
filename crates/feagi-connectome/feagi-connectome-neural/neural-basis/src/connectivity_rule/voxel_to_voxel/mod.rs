use crate::connectivity_rule::{AxonBundleSynapseLocalCount, ConnectivityRule};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use feagi_basis::prelude::{QuantizedUnsignedIntegerTrait, SpatialCoordinate};
use feagi_basis::voxels::CorticalAreaVoxelCoordinates;
use crate::neuron_model::neuron_layout::NeuronLayoutRestrictor;

//pub mod vector;
pub mod all_to_voxel;
pub mod voxel_to_all;
pub mod single_voxel;

/// Connectivity rule that connects a voxel layout area to a voxel layout area
pub trait ConnectivityRuleVoxelToVoxel<FIQ: FeagiIndexQuantization>:
ConnectivityRule<
    FIQ,
    MappingInputLayout=NeuronLayoutVoxel<FIQ>,
    MappingOutputLayout=NeuronLayoutVoxel<FIQ>
>
{
    type Generator<'a>: ConnectivityGeneratorVoxelToVoxel<FIQ> where Self: 'a;

    type ValidationError: core::error::Error;

    // NOTE: Take ownership (copy) of the dims since it would otherwise have a separate lifetime
    // which would be annoying to deal with

    fn spawn_generator
    (
        &self,
        source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
        destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    ) -> Result<Self::Generator<'_>, Self::ValidationError>;
}

/// Connectivity generator that connects a voxel layout area to a voxel layout area
pub trait ConnectivityGeneratorVoxelToVoxel<FIQ: FeagiIndexQuantization> {
    fn source_dims(&self) -> &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;

    fn destination_dims(&self) -> &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>;

    fn number_of_neuron_pairs(&self) -> &AxonBundleSynapseLocalCount<FIQ::SynapseIndexCountQuant>;

    /// How many of the rule's elements do not fit the given dimensions, and so generate no pairs
    fn number_of_skipped_elements(&self) -> usize;

    fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item=LocalNeuronIndexPair<FIQ>>;
}


pub type LocalNeuronIndexPair<FIQ: FeagiIndexQuantization> = (
    CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
);

fn voxel_fits_within<FIQ: FeagiIndexQuantization>(
    voxel: &CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    dims: &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
) -> bool {
    // Zipping stops after XYZ, since voxels have no density axis
    voxel.as_slice().iter().zip(dims.as_slice().iter())
        .all(|(position, size)| position.quant_to_usize() < size.quant_to_usize())
}

/// Adds a density position to a voxel, giving the coordinates of one neuron within it
fn neuron_coordinates<FIQ: FeagiIndexQuantization>(
    voxel: &CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    density_position: usize,
) -> DimensionalCorticalAreaNeuronCoordinates<FIQ::NeuronIndexQuant> {
    let [x, y, z] = voxel.as_slice();
    SpatialCoordinate::new_coordinate([
        x.deref(),
        y.deref(),
        z.deref(),
        FIQ::NeuronIndexQuant::quant_from_usize_unchecked(density_position),
    ])
}