use crate::connectivity_rule::voxel_to_voxel::{neuron_coordinates, voxel_fits_within, ConnectivityGeneratorVoxelToVoxel, ConnectivityRuleVoxelToVoxel, LocalNeuronIndexPair};
use crate::connectivity_rule::{AxonBundleSynapseLocalCount, ConnectivityRule, ExactCountIterator};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::neuron_model::neuron_layout::NeuronLayout;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, DimensionalCorticalAreaDimensions};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_basis::voxels::CorticalAreaVoxelCoordinates;

//region Rule

/// Connectivity Rule (Voxel to Voxel) that connects every neuron of the source area to every
/// neuron of a single given voxel in the destination area
pub struct ConnectivityRuleAllToVoxel<FIQ: FeagiIndexQuantization> {
    destination: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleAllToVoxel<FIQ> {
    pub fn new(destination: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>) -> Self {
        Self { destination }
    }

    pub fn destination(&self) -> &CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant> {
        &self.destination
    }
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRule<FIQ> for ConnectivityRuleAllToVoxel<FIQ> {
    type MappingInputLayout = NeuronLayoutVoxel<FIQ>;

    type MappingOutputLayout = NeuronLayoutVoxel<FIQ>;
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVoxelToVoxel<FIQ> for ConnectivityRuleAllToVoxel<FIQ> {
    type Generator<'a> = ConnectivityGeneratorAllToVoxel<'a, FIQ>;

    type ValidationError = ConnectivityGeneratorAllToVoxelError;

    fn spawn_generator(
        &self,
        source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
        destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    ) -> Result<Self::Generator<'_>, Self::ValidationError> {
        if !voxel_fits_within::<FIQ>(&self.destination, &destination_dims) {
            return Err(ConnectivityGeneratorAllToVoxelError::DestinationOutOfBounds);
        }

        let source_neurons = NeuronLayoutVoxel::<FIQ>::new(source_dims)
            .get_number_neurons_to_allocate_for()
            .quant_to_usize();
        let destination_density = destination_dims.as_slice()[3].quant_to_usize();
        let total_pairs = source_neurons.checked_mul(destination_density)
            .ok_or(ConnectivityGeneratorAllToVoxelError::TooManySynapses)?;
        let number_pairs = AxonBundleSynapseLocalCount::quant_try_from_usize(total_pairs)
            .map_err(|_| ConnectivityGeneratorAllToVoxelError::TooManySynapses)?;

        Ok(ConnectivityGeneratorAllToVoxel {
            source_dims,
            destination_dims,
            destination: &self.destination,
            source_neurons,
            number_pairs,
        })
    }
}

//endregion

//region Generator

/// Connectivity Generator (Voxel to Voxel) that connects every neuron of the source area to every
/// neuron of a single voxel in the destination area. The voxel is validated to fit on creation
pub struct ConnectivityGeneratorAllToVoxel<'a, FIQ: FeagiIndexQuantization> {
    source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    destination: &'a CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    source_neurons: usize,
    number_pairs: AxonBundleSynapseLocalCount<FIQ::SynapseIndexCountQuant>,
}

impl<'a, FIQ: FeagiIndexQuantization> ConnectivityGeneratorVoxelToVoxel<FIQ> for ConnectivityGeneratorAllToVoxel<'a, FIQ> {

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
        // The single destination voxel is checked when spawning, so nothing is ever skipped
        0
    }

    fn iter(&self) -> impl ExactSizeIterator<Item=LocalNeuronIndexPair<FIQ>> {
        let destination_layout = NeuronLayoutVoxel::<FIQ>::new(self.destination_dims);
        let destination_density = self.destination_dims.as_slice()[3].quant_to_usize();
        let destination = self.destination;

        let pairs = (0..self.source_neurons).flat_map(move |source_index| {
            let source = CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(source_index);
            (0..destination_density).map(move |destination_d| {
                (source, destination_layout.neuron_coordinates_to_index(
                    &neuron_coordinates::<FIQ>(destination, destination_d),
                ))
            })
        });

        ExactCountIterator::new(pairs, self.number_pairs.quant_to_usize())
    }
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum ConnectivityGeneratorAllToVoxelError {
    #[error("Destination voxel lies outside the destination area dimensions")]
    DestinationOutOfBounds,
    #[error("Number of synapses to generate exceeds the synapse index quantization")]
    TooManySynapses,
}
