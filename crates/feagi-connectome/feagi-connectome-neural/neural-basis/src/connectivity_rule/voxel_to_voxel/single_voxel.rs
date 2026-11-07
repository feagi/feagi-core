use crate::connectivity_rule::voxel_to_voxel::{neuron_coordinates, voxel_fits_within, ConnectivityGeneratorVoxelToVoxel, ConnectivityRuleVoxelToVoxel, LocalNeuronIndexPair};
use crate::connectivity_rule::{AxonBundleSynapseLocalCount, ConnectivityRule, ExactCountIterator};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::neuron_model::neuron_layout::NeuronLayoutData;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_basis::voxels::CorticalAreaVoxelCoordinates;

//region Rule

/// Connectivity Rule (Voxel to Voxel) that connects every neuron of a single given voxel in the
/// source area to every neuron of a single given voxel in the destination area.
pub struct ConnectivityRuleSingleVoxel<FIQ: FeagiIndexQuantization> {
    source: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    destination: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleSingleVoxel<FIQ> {
    pub fn new(
        source: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
        destination: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    ) -> Self {
        Self { source, destination }
    }

    pub fn source(&self) -> &CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant> {
        &self.source
    }

    pub fn destination(&self) -> &CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant> {
        &self.destination
    }
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRule<FIQ> for ConnectivityRuleSingleVoxel<FIQ> {
    type MappingInputLayout = NeuronLayoutVoxel<FIQ>;

    type MappingOutputLayout = NeuronLayoutVoxel<FIQ>;
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVoxelToVoxel<FIQ> for ConnectivityRuleSingleVoxel<FIQ> {
    type Generator<'a> = ConnectivityGeneratorSingleVoxel<'a, FIQ>;

    type ValidationError = ConnectivityGeneratorSingleVoxelError;

    fn spawn_generator(
        &self,
        source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
        destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    ) -> Result<Self::Generator<'_>, Self::ValidationError> {
        if !voxel_fits_within::<FIQ>(&self.source, &source_dims) {
            return Err(ConnectivityGeneratorSingleVoxelError::SourceOutOfBounds);
        }
        if !voxel_fits_within::<FIQ>(&self.destination, &destination_dims) {
            return Err(ConnectivityGeneratorSingleVoxelError::DestinationOutOfBounds);
        }

        let source_density = source_dims.as_slice()[3].quant_to_usize();
        let destination_density = destination_dims.as_slice()[3].quant_to_usize();
        let total_pairs = source_density.checked_mul(destination_density)
            .ok_or(ConnectivityGeneratorSingleVoxelError::TooManySynapses)?;
        let number_pairs = AxonBundleSynapseLocalCount::quant_try_from_usize(total_pairs)
            .map_err(|_| ConnectivityGeneratorSingleVoxelError::TooManySynapses)?;

        Ok(ConnectivityGeneratorSingleVoxel {
            source_dims,
            destination_dims,
            source: &self.source,
            destination: &self.destination,
            number_pairs,
        })
    }
}

//endregion

//region Generator

/// Connectivity Generator (Voxel to Voxel) that connects every neuron of a single source voxel to
/// every neuron of a single destination voxel. Both voxels are validated to fit on creation.
pub struct ConnectivityGeneratorSingleVoxel<'a, FIQ: FeagiIndexQuantization> {
    source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    source: &'a CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    destination: &'a CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    number_pairs: AxonBundleSynapseLocalCount<FIQ::SynapseIndexCountQuant>,
}

impl<'a, FIQ: FeagiIndexQuantization> ConnectivityGeneratorVoxelToVoxel<FIQ> for ConnectivityGeneratorSingleVoxel<'a, FIQ> {

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
        // Both voxels are checked when spawning, so nothing is ever skipped
        0
    }

    fn iter(&self) -> impl ExactSizeIterator<Item=LocalNeuronIndexPair<FIQ>> {
        let source_layout = NeuronLayoutVoxel::<FIQ>::new(self.source_dims);
        let destination_layout = NeuronLayoutVoxel::<FIQ>::new(self.destination_dims);
        let source_density = self.source_dims.as_slice()[3].quant_to_usize();
        let destination_density = self.destination_dims.as_slice()[3].quant_to_usize();
        let (source_voxel, destination_voxel) = (self.source, self.destination);

        let pairs = (0..source_density).flat_map(move |source_d| {
            let source = source_layout.neuron_coordinates_to_index(
                &neuron_coordinates::<FIQ>(source_voxel, source_d),
            );
            (0..destination_density).map(move |destination_d| {
                (source, destination_layout.neuron_coordinates_to_index(
                    &neuron_coordinates::<FIQ>(destination_voxel, destination_d),
                ))
            })
        });

        ExactCountIterator::new(pairs, self.number_pairs.quant_to_usize())
    }
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum ConnectivityGeneratorSingleVoxelError {
    #[error("Source voxel lies outside the source area dimensions")]
    SourceOutOfBounds,
    #[error("Destination voxel lies outside the destination area dimensions")]
    DestinationOutOfBounds,
    #[error("Number of synapses to generate exceeds the synapse index quantization")]
    TooManySynapses,
}
