use crate::doublet::doublet_iterator::DoubletIterator;
use crate::neuron::cortical_area_layout::CorticalAreaLayoutDimensional;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::neurons::{DimensionCorticalArea4DCoordinate, NeuronCorticalLocalIndex, NeuronVoxelDensityIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;

/// Maps every neuron of a single source voxel (its full density column) to every neuron of the
/// destination cortical area.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DoubletIteratorOneToAll<FIQ: FeagiIndexQuantization> {
    /// Linear index of the source voxel at density 0
    source_base_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    /// Distance between two neurons of the same voxel at consecutive densities
    source_density_stride: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    source_density_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    destination_count: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    source_cursor: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    destination_cursor: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
}

impl<FIQ: FeagiIndexQuantization> DoubletIteratorOneToAll<FIQ> {
    pub fn new(
        source_voxel_coordinate: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
        source_layout: &CorticalAreaLayoutDimensional<FIQ>,
        destination_layout: &CorticalAreaLayoutDimensional<FIQ>,
    ) -> Self {
        let source_dimensions = source_layout.dimensions;
        let probe = DimensionCorticalArea4DCoordinate::new_from_voxel_and_density(
            source_voxel_coordinate,
            NeuronVoxelDensityIndex::QUANT_ZERO,
        );
        let destination_count = destination_layout.dimensions.number_contained_elements();
        if !source_dimensions.contains_coordinate(&probe)
            || destination_count == NeuronCorticalLocalIndex::QUANT_ZERO
        {
            return Self::empty();
        }

        // Linear indexes increment along x fastest, so the neurons of a single voxel are a full
        // xyz plane apart from each other rather than being contiguous.
        let source_density_stride = source_dimensions.get_x().deref()
            * source_dimensions.get_y().deref()
            * source_dimensions.get_z().deref();

        Self {
            source_base_index: source_dimensions.coordinate_to_linear_index_unchecked(probe),
            source_density_stride: NeuronCorticalLocalIndex::new(source_density_stride),
            source_density_count: NeuronCorticalLocalIndex::new(source_dimensions.get_d().deref()),
            destination_count,
            source_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
        }
    }

    /// An iterator that is already exhausted, for pairings that produce no synapses
    fn empty() -> Self {
        Self {
            source_base_index: NeuronCorticalLocalIndex::QUANT_ZERO,
            source_density_stride: NeuronCorticalLocalIndex::QUANT_ZERO,
            source_density_count: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_count: NeuronCorticalLocalIndex::QUANT_ZERO,
            source_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
            destination_cursor: NeuronCorticalLocalIndex::QUANT_ZERO,
        }
    }
}

impl<FIQ: FeagiIndexQuantization> Iterator for DoubletIteratorOneToAll<FIQ> {
    type Item = (
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.source_cursor >= self.source_density_count {
            return None;
        }
        let pair = (
            self.source_base_index + (self.source_cursor * self.source_density_stride),
            self.destination_cursor,
        );
        self.destination_cursor += NeuronCorticalLocalIndex::QUANT_ONE;
        if self.destination_cursor >= self.destination_count {
            self.destination_cursor = NeuronCorticalLocalIndex::QUANT_ZERO;
            self.source_cursor += NeuronCorticalLocalIndex::QUANT_ONE;
        }
        Some(pair)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Counted in usize rather than in the neuron quantization, as the total number of pairs
        // can exceed what a single neuron index is able to hold.
        let remaining_sources =
            self.source_density_count.quant_to_usize() - self.source_cursor.quant_to_usize();
        let remaining = (remaining_sources * self.destination_count.quant_to_usize())
            - self.destination_cursor.quant_to_usize();
        (remaining, Some(remaining))
    }
}

impl<FIQ: FeagiIndexQuantization> ExactSizeIterator for DoubletIteratorOneToAll<FIQ> {}

impl<FIQ: FeagiIndexQuantization> DoubletIterator<FIQ, CorticalAreaLayoutDimensional<FIQ>, CorticalAreaLayoutDimensional<FIQ>>
    for DoubletIteratorOneToAll<FIQ>
{
    fn get_number_of_synapses(&self) -> FIQ::NeuronIndexQuant {
        (self.source_density_count * self.destination_count).deref()
    }
}
