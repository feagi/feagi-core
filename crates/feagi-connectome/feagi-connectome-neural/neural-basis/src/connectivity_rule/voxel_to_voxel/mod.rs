use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
pub mod vector;

use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions, DimensionalCorticalAreaNeuronCoordinates};
use crate::connectivity_rule::{AxonBundleSynapseLocalCount, AxonBundleSynapseLocalIndex, ConnectivityRule, SynapseCount};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

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

    fn spawn_generator(
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



