use core::marker::PhantomData;
use crate::base_quantizable::QuantizableUIntType;
use crate::base_quantizable::QuantizableValueType;
use crate::neuron_voxels::descriptors::{
    NeuronVoxelCoordinate,
    NeuronVoxelDimensions,
    NeuronVoxelPotential,
    SingleCorticalNeuronVoxelCollectionType,
};
use crate::neuron_voxels::traits::{SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionSparse};

#[derive(Debug, Clone)]
pub struct NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    coord_x: Vec<CoordQuant>,
    coord_y: Vec<CoordQuant>,
    coord_z: Vec<CoordQuant>,
    potentials: Vec<NeuronVoxelPotential<VoxelPotentialQuant>>,
    _index_quant: PhantomData<NeuronVoxelIndexQuant>,
}



impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>, number_neurons_preallocated: NeuronVoxelIndexQuant) -> Self {
        Self {
            cortical_dimensions,
            coord_x: Vec::with_capacity(number_neurons_preallocated.to_usize()),
            coord_y: Vec::with_capacity(number_neurons_preallocated.to_usize()),
            coord_z: Vec::with_capacity(number_neurons_preallocated.to_usize()),
            potentials: Vec::with_capacity(number_neurons_preallocated.to_usize()),
            _index_quant: PhantomData
        }
    }

    /// Appends a single neuron voxel (coordinate + potential) to the collection.
    ///
    /// The coordinate is NOT validated against `cortical_dimensions`; callers that
    /// need bounds checking should do it prior to calling. This intentionally
    /// minimal insertion API exists primarily for deserialization, where every
    /// voxel has already been range-validated by whatever produced the bytes.
    #[inline]
    pub fn push_neuron_voxel_unchecked(
        &mut self,
        coordinate: NeuronVoxelCoordinate<CoordQuant>,
        potential: NeuronVoxelPotential<VoxelPotentialQuant>,
    ) {
        self.coord_x.push(coordinate.x);
        self.coord_y.push(coordinate.y);
        self.coord_z.push(coordinate.z);
        self.potentials.push(potential);
    }

    // ---- Structure-of-Arrays accessors ----
    //
    // These expose the underlying per-axis vectors so callers with bulk-copy
    // patterns (image-frame encoders, segmentors, serialization fast paths)
    // can push/extend directly. The accessors are purely additive; the
    // trait-based iter_coordinate / iter_index APIs remain the preferred
    // entry points for callers that don't need raw SoA access.

    #[inline]
    pub fn coord_x_slice(&self) -> &[CoordQuant] {
        &self.coord_x
    }

    #[inline]
    pub fn coord_y_slice(&self) -> &[CoordQuant] {
        &self.coord_y
    }

    #[inline]
    pub fn coord_z_slice(&self) -> &[CoordQuant] {
        &self.coord_z
    }

    #[inline]
    pub fn potentials_slice(&self) -> &[NeuronVoxelPotential<VoxelPotentialQuant>] {
        &self.potentials
    }

    /// Runs `f` with mutable borrows of the four underlying vectors. The
    /// vectors must stay the same length on exit; callers mutate by
    /// extending all four in lockstep. Length invariants are not checked
    /// here — encoders that rely on this pattern are expected to keep the
    /// SoA vectors aligned (this mirrors the pre-refactor
    /// `update_vectors_from_external` contract used by feagi-sensorimotor).
    #[inline]
    pub fn with_parts_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(
            &mut Vec<CoordQuant>,
            &mut Vec<CoordQuant>,
            &mut Vec<CoordQuant>,
            &mut Vec<NeuronVoxelPotential<VoxelPotentialQuant>>,
        ) -> R,
    {
        f(
            &mut self.coord_x,
            &mut self.coord_y,
            &mut self.coord_z,
            &mut self.potentials,
        )
    }

    /// Constructs a `NeuronVoxelCoordVector` directly from pre-built
    /// Structure-of-Arrays vectors. Vectors must share the same length,
    /// otherwise returns [`FeagiStructuresError::BadParameters`].
    #[cfg(feature = "alloc")]
    pub fn from_parts(
        cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
        coord_x: Vec<CoordQuant>,
        coord_y: Vec<CoordQuant>,
        coord_z: Vec<CoordQuant>,
        potentials: Vec<NeuronVoxelPotential<VoxelPotentialQuant>>,
    ) -> Result<Self, crate::FeagiStructuresError> {
        let n = potentials.len();
        if coord_x.len() != n || coord_y.len() != n || coord_z.len() != n {
            return Err(crate::FeagiStructuresError::BadParameters(
                format!(
                    "NeuronVoxelCoordVector::from_parts length mismatch: \
                     x={}, y={}, z={}, p={}",
                    coord_x.len(),
                    coord_y.len(),
                    coord_z.len(),
                    n
                ),
            ));
        }
        Ok(Self {
            cortical_dimensions,
            coord_x,
            coord_y,
            coord_z,
            potentials,
            _index_quant: PhantomData,
        })
    }
}



impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType = SingleCorticalNeuronVoxelCollectionType::CoordVector;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.cortical_dimensions.get_max_allowed_index_exclusive())
    }
}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.potentials.len())
    }

    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.potentials.capacity())
    }

    fn reserve(&mut self, number_of_neuron_voxels_to_reserve_for: NeuronVoxelIndexQuant) {
        self.potentials.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
        self.coord_x.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
        self.coord_y.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
        self.coord_z.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
    }

    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CoordQuant>) {
        self.clear_all_neurons();
        self.cortical_dimensions = new_dimensions;
    }

    fn shrink_to_fit(&mut self) {
        self.potentials.shrink_to_fit();
        self.coord_x.shrink_to_fit();
        self.coord_y.shrink_to_fit();
        self.coord_z.shrink_to_fit();
    }
}



impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionSparse<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn clear_all_neurons(&mut self) {
        self.potentials.clear();
        self.coord_x.clear();
        self.coord_y.clear();
        self.coord_z.clear();
    }

    fn iter_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        let dims = &self.cortical_dimensions;
        self.coord_x
            .iter()
            .zip(self.coord_y.iter())
            .zip(self.coord_z.iter())
            .zip(self.potentials.iter())
            .map(move |(((x, y), z), p)| {
                let c = NeuronVoxelCoordinate::new(*x, *y, *z);
                (dims.coordinate_to_linear_index(c), *p)
            })
    }

    fn iter_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.coord_x
            .iter()
            .zip(self.coord_y.iter())
            .zip(self.coord_z.iter())
            .zip(self.potentials.iter())
            .map(|(((x, y), z), p)| (NeuronVoxelCoordinate::new(*x, *y, *z), *p))
    }

    fn sort(&mut self) {
        let n = self.coord_x.len();
        if n <= 1 {
            return;
        }
        let mut order: Vec<usize> = (0..n).collect();
        let dims = &self.cortical_dimensions;
        order.sort_by_key(|&i| {
            dims.coordinate_to_linear_index::<NeuronVoxelIndexQuant>(NeuronVoxelCoordinate::new(
                self.coord_x[i],
                self.coord_y[i],
                self.coord_z[i],
            ))
            .to_usize()
        });
        let coord_x = core::mem::take(&mut self.coord_x);
        let coord_y = core::mem::take(&mut self.coord_y);
        let coord_z = core::mem::take(&mut self.coord_z);
        let potentials = core::mem::take(&mut self.potentials);
        self.coord_x = order.iter().map(|&i| coord_x[i]).collect();
        self.coord_y = order.iter().map(|&i| coord_y[i]).collect();
        self.coord_z = order.iter().map(|&i| coord_z[i]).collect();
        self.potentials = order.iter().map(|&i| potentials[i]).collect();
    }

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.iter_index()
    }

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.iter_coordinate()
    }
}