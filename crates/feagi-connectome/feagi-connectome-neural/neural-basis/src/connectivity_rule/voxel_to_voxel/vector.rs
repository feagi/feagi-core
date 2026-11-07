use crate::connectivity_rule::voxel_to_voxel::{ConnectivityGeneratorVoxelToVoxel, ConnectivityRuleVoxelToVoxel, LocalNeuronIndexPair};
use crate::connectivity_rule::{voxel_to_voxel, AxonBundleSynapseLocalCount, ConnectivityRule, ExactCountIterator};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutVoxel;
use crate::neuron_model::neuron_layout::NeuronLayoutData;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_basis::voxels::CorticalAreaVoxelCoordinates;


// TODO bad name with this being type vector! We need a store backing and a better name

//region Rule

/// Connectivity Rule (Voxel to Voxel) that contains a list of voxel coordinate pairs where the
/// given source voxel connects to the destination voxel. All neurons in each voxel are connected.
pub struct ConnectivityRuleVector<FIQ: FeagiIndexQuantization> {
    elements: Vec<ConnectivityRuleVectorElement<FIQ>>,
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVector<FIQ> {

    #[cfg(feature = "std")]
    pub fn new(elements: Vec<ConnectivityRuleVectorElement<FIQ>>) -> Result<Self, ConnectivityRuleError> {
        let mut seen = std::collections::HashSet::with_capacity(elements.len());
        if !elements.iter().all(|element| seen.insert((&element.source, &element.destination))) {
            return Err(ConnectivityRuleError::DuplicateMappings);
        }

        Ok(Self { elements })
    }
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRule<FIQ>
    for ConnectivityRuleVector<FIQ>
{
    type MappingInputLayout = NeuronLayoutVoxel<FIQ>;

    type MappingOutputLayout = NeuronLayoutVoxel<FIQ>;
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVoxelToVoxel<FIQ> for ConnectivityRuleVector<FIQ> {
    type Generator<'a> = ConnectivityGeneratorVector<'a, FIQ>;

    type ValidationError = ConnectivityGeneratorError;

    fn spawn_generator(
        &self,
        source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
        destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    ) -> Result<Self::Generator<'_>, Self::ValidationError> {
        let number_valid = self.elements.iter()
            .filter(|element| element.fits_within(&source_dims, &destination_dims))
            .count();

        let source_density = source_dims.as_slice()[3].quant_to_usize();
        let destination_density = destination_dims.as_slice()[3].quant_to_usize();
        let total_pairs = number_valid.checked_mul(source_density)
            .and_then(|pairs| pairs.checked_mul(destination_density))
            .ok_or(ConnectivityGeneratorError::TooManySynapses)?;
        let number_pairs = AxonBundleSynapseLocalCount::quant_try_from_usize(total_pairs)
            .map_err(|_| ConnectivityGeneratorError::TooManySynapses)?;

        Ok(ConnectivityGeneratorVector {
            source_dims,
            destination_dims,
            pairs_unfiltered: &self.elements,
            number_pairs,
            number_skipped: self.elements.len() - number_valid,
        })
    }
}

//endregion

//region Generator

/// Connectivity Generator (Voxel to Voxel) that contains a list of voxel coordinate pairs where the
/// given source voxel connects to the destination voxel. All neurons in each voxel are connected.
pub struct ConnectivityGeneratorVector<'a, FIQ: FeagiIndexQuantization> {
    source_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    destination_dims: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    pairs_unfiltered: &'a [ConnectivityRuleVectorElement<FIQ>],
    number_pairs: AxonBundleSynapseLocalCount<FIQ::SynapseIndexCountQuant>,
    number_skipped: usize,
}

impl<'a, FIQ: FeagiIndexQuantization> ConnectivityGeneratorVoxelToVoxel<FIQ> for ConnectivityGeneratorVector<'a, FIQ> {

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
        self.number_skipped
    }

    fn iter(&self) -> impl ExactSizeIterator<Item=LocalNeuronIndexPair<FIQ>> {
        let (source_dims, destination_dims) = (self.source_dims, self.destination_dims);
        let source_layout = NeuronLayoutVoxel::<FIQ>::new(source_dims);
        let destination_layout = NeuronLayoutVoxel::<FIQ>::new(destination_dims);
        let source_density = source_dims.as_slice()[3].quant_to_usize();
        let destination_density = destination_dims.as_slice()[3].quant_to_usize();

        let pairs = self.pairs_unfiltered.iter()
            .filter(move |element| element.fits_within(&source_dims, &destination_dims))
            .flat_map(move |element| {
                (0..source_density).flat_map(move |source_d| {
                    let source = source_layout.neuron_coordinates_to_index(
                        &voxel_to_voxel::neuron_coordinates::<FIQ>(&element.source, source_d),
                    );
                    (0..destination_density).map(move |destination_d| {
                        (source, destination_layout.neuron_coordinates_to_index(
                            &voxel_to_voxel::neuron_coordinates::<FIQ>(&element.destination, destination_d),
                        ))
                    })
                })
            });

        ExactCountIterator::new(pairs, self.number_pairs.quant_to_usize())
    }
}

//endregion

//region Element
pub struct ConnectivityRuleVectorElement<FIQ: FeagiIndexQuantization> {
    pub source: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
    pub destination: CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> ConnectivityRuleVectorElement<FIQ> {
    /// Whether both voxels fit their dimensions. Decides which elements generate pairs, so
    /// anything that counts or iterates those elements must use this.
    fn fits_within(
        &self,
        source_dims: &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
        destination_dims: &DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>,
    ) -> bool {
        voxel_to_voxel::voxel_fits_within::<FIQ>(&self.source, source_dims)
            && voxel_to_voxel::voxel_fits_within::<FIQ>(&self.destination, destination_dims)
    }
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum ConnectivityRuleError {
    #[error("Duplicate Mappings detected")]
    DuplicateMappings
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectivityGeneratorError {
    #[error("Number of synapses to generate exceeds the synapse index quantization")]
    TooManySynapses,
}


