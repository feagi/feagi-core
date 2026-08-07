use crate::cortical_mapping_entry::doublet::doublet_iterator::DoubletIterator;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::neurons::{DimensionalCorticalArea4DCoordinate, NeuronCorticalLocalIndex, NeuronVoxelDensityIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;
use crate::cortical_area::cortical_area_layout::implementations::dimensional::CorticalAreaLayoutDimensional;

/// Maps every neuron of the source cortical area to every neuron of a single destination voxel
/// (its full density column). The inverse of
/// [`DoubletIteratorOneToAll`](crate::cortical_mapping_entry::doublet::implementations::one_to_all::DoubletIteratorOneToAll).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DoubletIteratorAllToOne<FIQ: FeagiIndexQuantization> {
    /// Linear index of the destination voxel at density 0
    destination_base_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    /// Distance between two neurons of the same voxel at consecutive densities
    destination_density_stride: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    destination_density_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    source_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    source_cursor: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    destination_cursor: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> DoubletIteratorAllToOne<FIQ> {
    pub fn new(
        destination_voxel_coordinate: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
        source_layout: &CorticalAreaLayoutDimensional<FIQ>,
        destination_layout: &CorticalAreaLayoutDimensional<FIQ>,
    ) -> Self {
        let destination_dimensions = destination_layout.dimensions;
        let probe = DimensionalCorticalArea4DCoordinate::new_from_voxel_and_density(
            destination_voxel_coordinate,
            NeuronVoxelDensityIndex::QUANT_ZERO,
        );
        let source_count = source_layout.dimensions.number_contained_elements();
        if !destination_dimensions.contains_coordinate(&probe)
            || source_count == NeuronCorticalLocalIndex::QUANT_ZERO
        {
            return Self::empty();
        }

        // Linear indexes increment along x fastest, so the neurons of a single voxel are a full
        // xyz plane apart from each other rather than being contiguous.
        let destination_density_stride = destination_dimensions.get_x().deref()
            * destination_dimensions.get_y().deref()
            * destination_dimensions.get_z().deref();

        Self {
            destination_base_index: destination_dimensions.coordinate_to_linear_index_unchecked(probe),
            destination_density_stride: NeuronCorticalLocalIndex::new(destination_density_stride),
            destination_density_count: NeuronCorticalLocalIndex::new(destination_dimensions.get_d().deref()),
            source_count,
            source_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
        }
    }

    /// An iterator that is already exhausted, for pairings that produce no synapses
    fn empty() -> Self {
        Self {
            destination_base_index: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_density_stride: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_density_count: NeuronCorticalLocalIndex::QUANT_ZERO,
            source_count: NeuronCorticalLocalIndex::QUANT_ZERO,
            source_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
        }
    }
}

impl<FIQ: FeagiIndexQuantization> Iterator for DoubletIteratorAllToOne<FIQ> {
    type Item = (
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.source_cursor >= self.source_count {
            return None;
        }
        let pair = (
            self.source_cursor,
            self.destination_base_index + (self.destination_cursor * self.destination_density_stride),
        );
        self.destination_cursor += NeuronCorticalLocalIndex::QUANT_ONE;
        if self.destination_cursor >= self.destination_density_count {
            self.destination_cursor = NeuronCorticalLocalIndex::QUANT_ZERO;
            self.source_cursor += NeuronCorticalLocalIndex::QUANT_ONE;
        }
        Some(pair)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Counted in usize rather than in the neuron quantization, as the total number of pairs
        // can exceed what a single neuron index is able to hold.
        let remaining_sources = self.source_count.quant_to_usize() - self.source_cursor.quant_to_usize();
        let remaining = (remaining_sources * self.destination_density_count.quant_to_usize())
            - self.destination_cursor.quant_to_usize();
        (remaining, Some(remaining))
    }
}

impl<FIQ: FeagiIndexQuantization> ExactSizeIterator for DoubletIteratorAllToOne<FIQ> {}

impl<FIQ: FeagiIndexQuantization> DoubletIterator<FIQ, CorticalAreaLayoutDimensional<FIQ>, CorticalAreaLayoutDimensional<FIQ>>
    for DoubletIteratorAllToOne<FIQ>
{
    const CAN_BE_RECOMPUTED_FOR_CORTICAL_RESIZING: bool = true;

    fn get_number_of_synapses(&self) -> FIQ::NeuronIndexQuant {
        (self.source_count * self.destination_density_count).deref()
    }
}
