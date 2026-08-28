use feagi_data::neurons::voxel_potentials::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::spatial::integer_signed::SignedCoordinate3D;

// TODO complete this after the changes as highlighted in https://github.com/feagi/feagi-core/issues/236
// right now adding negative numbers is weird

/*
/// Maps all source dimensional neurons to destination dimensional neurons by coordinate + a linear
/// offset
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DoubleIteratorSingleCoordinateOffset<FIQ: FeagiIndexQuantization>{
    number_of_doublets_to_be_generated: FIQ::NeuronIndexQuant,
    coordinate_cursor: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
    coordinate_offset: SignedCoordinate3D<>,
}

impl< FIQ: FeagiIndexQuantization> DoubleIteratorSingleCoordinateOffset<FIQ> {




}

 */
