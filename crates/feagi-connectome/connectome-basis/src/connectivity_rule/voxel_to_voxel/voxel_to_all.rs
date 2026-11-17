use crate::connectivity_rule::voxel_to_voxel::{neuron_index, voxel_fits_within, voxel_index_helper, ConnectivityGeneratorVoxelToVoxel, ConnectivityRuleVoxelToVoxel, LocalNeuronIndexPair};
use crate::connectivity_rule::{AxonBundleSynapseLocalCount, ConnectivityRule, ExactCountIterator};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_basis::voxels::CorticalAreaVoxelCoordinates;

//region Rule

/// Connectivity Rule (Voxel to Voxel) that connects every neuron of a single given voxel in the
/// source area to every neuron of the destination area.
pub struct ConnectivityRuleVoxelToAll<FIQ: FeagiIndexQuantization> {
    source: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVoxelToAll<FIQ> {
    pub fn new(source: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>) -> Self {
        Self { source }
    }

    pub fn source(&self) -> &CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant> {
        &self.source
    }
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRule<FIQ> for ConnectivityRuleVoxelToAll<FIQ> {
    type MappingInputLayout = NeuronLayoutVoxel<FIQ>;

    type MappingOutputLayout = NeuronLayoutVoxel<FIQ>;
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVoxelToVoxel<FIQ> for ConnectivityRuleVoxelToAll<FIQ> {
    type Generator<'a> = ConnectivityGeneratorVoxelToAll<'a, FIQ>;

    type ValidationError = ConnectivityGeneratorVoxelToAllError;

    fn spawn_generator(
        &self,
        source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
        destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    ) -> Result<Self::Generator<'_>, Self::ValidationError> {
        if !voxel_fits_within::<FIQ>(&self.source, &source_dims) {
            return Err(ConnectivityGeneratorVoxelToAllError::SourceOutOfBounds);
        }

        let source_density = source_dims.as_slice()[3].quant_to_usize();
        let destination_neurons = destination_dims.spatial_element_count();
        let total_pairs = source_density.checked_mul(destination_neurons)
            .ok_or(ConnectivityGeneratorVoxelToAllError::TooManySynapses)?;
        let number_pairs = AxonBundleSynapseLocalCount::quant_try_from_usize(total_pairs)
            .map_err(|_| ConnectivityGeneratorVoxelToAllError::TooManySynapses)?;

        Ok(ConnectivityGeneratorVoxelToAll {
            source_dims,
            destination_dims,
            source: &self.source,
            destination_neurons,
            number_pairs,
        })
    }
}

//endregion

//region Generator

/// Connectivity Generator (Voxel to Voxel) that connects every neuron of a single voxel in the
/// source area to every neuron of the destination area. The voxel is validated to fit on creation.
pub struct ConnectivityGeneratorVoxelToAll<'a, FIQ: FeagiIndexQuantization> {
    source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    source: &'a CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    destination_neurons: usize,
    number_pairs: AxonBundleSynapseLocalCount<FIQ::SynapseIndexCountQuant>,
}

impl<'a, FIQ: FeagiIndexQuantization> ConnectivityGeneratorVoxelToVoxel<FIQ> for ConnectivityGeneratorVoxelToAll<'a, FIQ> {

    fn source_dims(&self) -> &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant> {
        &self.source_dims
    }

    fn destination_dims(&self) -> &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant> {
        &self.destination_dims
    }

    fn number_of_neuron_pairs(&self) -> &AxonBundleSynapseLocalCount<FIQ::SynapseIndexCountQuant> {
        &self.number_pairs
    }

    fn number_of_skipped_elements(&self) -> usize {
        // The single source voxel is checked when spawning, so nothing is ever skipped
        0
    }

    fn iter(&self) -> impl ExactSizeIterator<Item=LocalNeuronIndexPair<FIQ>> {
        let source_layout = voxel_index_helper::<FIQ>(self.source_dims);
        let source_density = self.source_dims.as_slice()[3].quant_to_usize();
        let destination_neurons = self.destination_neurons;
        let source_voxel = self.source;

        let pairs = (0..source_density).flat_map(move |source_d| {
            let source = neuron_index::<FIQ>(&source_layout, source_voxel, source_d);
            (0..destination_neurons).map(move |destination_index| {
                (source, CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(destination_index))
            })
        });

        ExactCountIterator::new(pairs, self.number_pairs.quant_to_usize())
    }
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum ConnectivityGeneratorVoxelToAllError {
    #[error("Source voxel lies outside the source area dimensions")]
    SourceOutOfBounds,
    #[error("Number of synapses to generate exceeds the synapse index quantization")]
    TooManySynapses,
}
