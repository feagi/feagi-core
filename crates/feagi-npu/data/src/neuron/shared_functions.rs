

// TODO a lot of these should have additional bounds checking when in debug mode


/// Converts an index over a set of dimensions into a coordinate
pub fn relative_index_to_coordinate<Quant>(relative_index: &Quant, dimensions: &NeuronVoxelDimensions<Quant>, neurons_per_voxel: Quant) -> NeuronVoxelCoordinates<Quant> where
    Quant: QuantizableUInt,
{
    // TODO debug bounds checking
    let div_index = relative_index / neurons_per_voxel;
    let x = div_index % dimensions.x;
    let y = (div_index / dimensions.x) % dimensions.y;
    let z = div_index / (dimensions.x * dimensions.y);
    NeuronVoxelCoordinates::new(x, y, z)
}

/// Converts an index over a set of dimensions into a coordinate in place
pub fn relative_index_to_coordinate_in_place<Quant>(relative_index: &Quant, dimensions: &NeuronVoxelDimensions<Quant>, neurons_per_voxel: Quant, out: &mut NeuronVoxelCoordinates<Quant>) where
    Quant: QuantizableUInt,
{
    // TODO debug bounds checking
    let div_index = relative_index / neurons_per_voxel;
    out.x = div_index % dimensions.x;
    out.y = (div_index / dimensions.x) % dimensions.y;
    out.z = div_index / (dimensions.x * dimensions.y);
}

/// Converts a coordinate over a set of dimensions into an index
pub fn coordinate_to_relative_index<Quant>(coordinate: &NeuronVoxelCoordinates<Quant>, dimensions: &NeuronVoxelDimensions<Quant>, neurons_per_voxel: Quant) -> Quant where
    Quant: QuantizableUInt,
{
    // TODO debug bounds checking
    (coordinate.x + (coordinate.y * dimensions.x) + (coordinate.z * dimensions.x * dimensions.y)) * neurons_per_voxel
}

/// Converts a coordinate over a set of dimensions into an index in place
pub fn coordinate_to_relative_index_in_place<Quant>(coordinate: &NeuronVoxelCoordinates<Quant>, dimensions: &NeuronVoxelDimensions<Quant>, neurons_per_voxel: Quant, out: &mut Quant) where
    Quant: QuantizableUInt,
{
    // TODO debug bounds checking
    *out = (coordinate.x + (coordinate.y * dimensions.x) + (coordinate.z * dimensions.x * dimensions.y)) * neurons_per_voxel;
}

/// Converts a relative index iterator over a set of dimensions into a coordinate iterator
pub fn relative_index_iterator_to_coordinate_iterator<'a, Quant, RelativeIndexIterator>(
    relative_indexes: RelativeIndexIterator,
    dimensions: &'a NeuronVoxelDimensions<Quant>,
    neurons_per_voxel: Quant,
) -> impl Iterator<Item = NeuronVoxelCoordinates<Quant>> + 'a
where
    Quant: QuantizableUInt + 'a,
    RelativeIndexIterator: IntoIterator<Item = &'a Quant> + 'a,
{
    relative_indexes
        .into_iter()
        .map(move |relative_index| relative_index_to_coordinate(relative_index, dimensions, neurons_per_voxel))
}

/// Converts a coordinate iterator over a set of dimensions into a relative index iterator
pub fn coordinate_iterator_to_relative_index_iterator<'a, Quant, CoordinateIterator>(
    coordinates: CoordinateIterator,
    dimensions: &'a NeuronVoxelDimensions<Quant>,
    neurons_per_voxel: Quant,
) -> impl Iterator<Item = Quant> + 'a
where
    Quant: QuantizableUInt + 'a,
    CoordinateIterator: IntoIterator<Item = &'a NeuronVoxelCoordinates<Quant>> + 'a,
{
    coordinates
        .into_iter()
        .map(move |coordinate| coordinate_to_relative_index(coordinate, dimensions, neurons_per_voxel))
}

/// Parallel equivalent of `relative_index_iterator_to_coordinate_iterator`
#[cfg(feature = "rayon")]
pub fn relative_index_par_iterator_to_coordinate_par_iterator<'a, Quant, RelativeIndexParallelIterator>(
    relative_indexes: RelativeIndexParallelIterator,
    dimensions: &'a NeuronVoxelDimensions<Quant>,
    neurons_per_voxel: Quant,
) -> impl rayon::iter::ParallelIterator<Item = NeuronVoxelCoordinates<Quant>> + 'a
where
    Quant: QuantizableUInt + Send + Sync + 'a,
    RelativeIndexParallelIterator: rayon::iter::IntoParallelIterator<Item = &'a Quant> + 'a,
{
    rayon::iter::IntoParallelIterator::into_par_iter(relative_indexes)
        .map(move |relative_index| relative_index_to_coordinate(relative_index, dimensions, neurons_per_voxel))
}

/// Parallel equivalent of `coordinate_iterator_to_relative_index_iterator`
#[cfg(feature = "rayon")]
pub fn coordinate_par_iterator_to_relative_index_par_iterator<'a, Quant, CoordinateParallelIterator>(
    coordinates: CoordinateParallelIterator,
    dimensions: &'a NeuronVoxelDimensions<Quant>,
    neurons_per_voxel: Quant,
) -> impl rayon::iter::ParallelIterator<Item = Quant> + 'a
where
    Quant: QuantizableUInt + Send + Sync + 'a,
    CoordinateParallelIterator: rayon::iter::IntoParallelIterator<Item = &'a NeuronVoxelCoordinates<Quant>> + 'a,
{
    rayon::iter::IntoParallelIterator::into_par_iter(coordinates)
        .map(move |coordinate| coordinate_to_relative_index(coordinate, dimensions, neurons_per_voxel))
}

/// Builds a coordinate iterator over `[0..x) x [0..y) x [0..z)` with uniform gap spacing.
///
/// A `gap_spacing` of `0` means no skipped coordinates (step size = 1).
/// A `gap_spacing` of `1` means every other coordinate is yielded (step size = 2), etc.
pub fn coordinate_iterator_with_gap_spacing<Quant>(
    x: Quant,
    y: Quant,
    z: Quant,
    gap_spacing: Quant,
) -> impl Iterator<Item = NeuronVoxelCoordinates<Quant>>
where
    Quant: QuantizableUInt,
{
    let x_max = x.to_usize();
    let y_max = y.to_usize();
    let z_max = z.to_usize();
    let step = gap_spacing.to_usize().saturating_add(1);

    (0..z_max).step_by(step).flat_map(move |z_coord| {
        (0..y_max).step_by(step).flat_map(move |y_coord| {
            (0..x_max).step_by(step).map(move |x_coord| {
                NeuronVoxelCoordinates::new(
                    Quant::from_usize(x_coord),
                    Quant::from_usize(y_coord),
                    Quant::from_usize(z_coord),
                )
            })
        })
    })
}

/// Rayon parallel equivalent of `coordinate_iterator_with_gap_spacing`.
#[cfg(feature = "rayon")]
pub fn coordinate_par_iterator_with_gap_spacing<Quant>(
    x: Quant,
    y: Quant,
    z: Quant,
    gap_spacing: Quant,
) -> impl rayon::iter::ParallelIterator<Item = NeuronVoxelCoordinates<Quant>>
where
    Quant: QuantizableUInt + Send + Sync,
{
    let x_max = x.to_usize();
    let y_max = y.to_usize();
    let z_max = z.to_usize();
    let step = gap_spacing.to_usize().saturating_add(1);

    let xy = x_max.saturating_mul(y_max);
    let xyz = xy.saturating_mul(z_max);

    rayon::iter::IntoParallelIterator::into_par_iter(0..xyz).filter_map(move |linear_index| {
        let x_coord = linear_index % x_max;
        let y_coord = (linear_index / x_max) % y_max;
        let z_coord = linear_index / xy;

        if x_coord % step == 0 && y_coord % step == 0 && z_coord % step == 0 {
            Some(NeuronVoxelCoordinates::new(
                Quant::from_usize(x_coord),
                Quant::from_usize(y_coord),
                Quant::from_usize(z_coord),
            ))
        } else {
            None
        }
    })
}

/// Builds an index iterator over a relative index range while skipping entries based on
/// voxel-coordinate gap spacing per axis.
///
/// A gap of `0` means no skip on that axis (step size = 1).
pub fn relative_index_range_iterator_with_xyz_gap<Quant>(
    relative_index_range: core::ops::Range<Quant>,
    dimensions: &NeuronVoxelDimensions<Quant>,
    neurons_per_voxel: Quant,
    gap_x: Quant,
    gap_y: Quant,
    gap_z: Quant,
) -> impl Iterator<Item = Quant>
where
    Quant: QuantizableUInt,
{
    let start = relative_index_range.start.to_usize();
    let end = relative_index_range.end.to_usize();

    let dim_x = dimensions.x.to_usize();
    let dim_y = dimensions.y.to_usize();
    let neurons_per_voxel_usize = neurons_per_voxel.to_usize();

    let step_x = gap_x.to_usize().saturating_add(1);
    let step_y = gap_y.to_usize().saturating_add(1);
    let step_z = gap_z.to_usize().saturating_add(1);

    let xy = dim_x.saturating_mul(dim_y);

    (start..end).filter_map(move |relative_index| {
        // TODO debug bounds checking
        let div_index = relative_index / neurons_per_voxel_usize;
        let x_coord = div_index % dim_x;
        let y_coord = (div_index / dim_x) % dim_y;
        let z_coord = div_index / xy;

        if x_coord % step_x == 0 && y_coord % step_y == 0 && z_coord % step_z == 0 {
            Some(Quant::from_usize(relative_index))
        } else {
            None
        }
    })
}

/// Rayon parallel equivalent of `relative_index_range_iterator_with_xyz_gap`.
#[cfg(feature = "rayon")]
pub fn relative_index_range_par_iterator_with_xyz_gap<Quant>(
    relative_index_range: core::ops::Range<Quant>,
    dimensions: &NeuronVoxelDimensions<Quant>,
    neurons_per_voxel: Quant,
    gap_x: Quant,
    gap_y: Quant,
    gap_z: Quant,
) -> impl rayon::iter::ParallelIterator<Item = Quant>
where
    Quant: QuantizableUInt + Send + Sync,
{
    let start = relative_index_range.start.to_usize();
    let end = relative_index_range.end.to_usize();

    let dim_x = dimensions.x.to_usize();
    let dim_y = dimensions.y.to_usize();
    let neurons_per_voxel_usize = neurons_per_voxel.to_usize();

    let step_x = gap_x.to_usize().saturating_add(1);
    let step_y = gap_y.to_usize().saturating_add(1);
    let step_z = gap_z.to_usize().saturating_add(1);

    let xy = dim_x.saturating_mul(dim_y);

    rayon::iter::IntoParallelIterator::into_par_iter(start..end).filter_map(move |relative_index| {
        // TODO debug bounds checking
        let div_index = relative_index / neurons_per_voxel_usize;
        let x_coord = div_index % dim_x;
        let y_coord = (div_index / dim_x) % dim_y;
        let z_coord = div_index / xy;

        if x_coord % step_x == 0 && y_coord % step_y == 0 && z_coord % step_z == 0 {
            Some(Quant::from_usize(relative_index))
        } else {
            None
        }
    })
}