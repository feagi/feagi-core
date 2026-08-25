use crate::neurons::voxel_potentials::wrapped_values::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelLinearIndex};
use crate::values::quantizable::{
    QuantizedDecimalTrait,
    QuantizedUnsignedIntegerTrait,
    QuantizedUnsignedIntegerUnwrappedTrait,
};

/// Dense X/Y/Z packed voxel storage with x-axis  in linear order.
pub struct VoxelVector<QI: QuantizedUnsignedIntegerUnwrappedTrait, QD: QuantizedDecimalTrait> {
    dimensions: NeuronVoxelDimensions<QI>,
    voxels: Vec<QD>,
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, QD: QuantizedDecimalTrait> VoxelVector<QI, QD> {
    /// Creates a dense voxel vector for `dimensions`, initialized to `QD::QUANT_ZERO`.
    pub fn new_zeroed(dimensions: NeuronVoxelDimensions<QI>) -> Self {
        let voxel_count = dimensions.number_contained_elements().quant_to_usize();
        Self {
            dimensions,
            voxels: vec![QD::QUANT_ZERO; voxel_count],
        }
    }

    /// Returns the dimensions this vector is laid out against.
    pub fn dimensions(&self) -> &NeuronVoxelDimensions<QI> {
        &self.dimensions
    }

    /// Returns the total number of packed voxels.
    pub fn len(&self) -> usize {
        self.voxels.len()
    }

    /// Returns `true` when there are no voxels.
    pub fn is_empty(&self) -> bool {
        self.voxels.is_empty()
    }

    /// Returns `true` if the provided linear index is within bounds.
    pub fn contains_linear_index(&self, index: NeuronVoxelLinearIndex<QI>) -> bool {
        self.dimensions.contains_linear_index(index)
    }

    /// Returns `true` if the provided coordinate is within bounds.
    pub fn contains_coordinate(&self, coordinate: &NeuronVoxelCoordinate<QI>) -> bool {
        self.dimensions.contains_coordinate(coordinate)
    }

    /// Gets a voxel by linear index.
    pub fn get_by_linear_index(&self, index: NeuronVoxelLinearIndex<QI>) -> Option<&QD> {
        if !self.contains_linear_index(index) {
            return None;
        }
        self.voxels.get(index.deref().quant_to_usize())
    }

    /// Mutably gets a voxel by linear index.
    pub fn get_by_linear_index_mut(&mut self, index: NeuronVoxelLinearIndex<QI>) -> Option<&mut QD> {
        if !self.contains_linear_index(index) {
            return None;
        }
        self.voxels.get_mut(index.deref().quant_to_usize())
    }

    /// Sets a voxel by linear index, returning the old value when in bounds.
    pub fn set_by_linear_index(&mut self, index: NeuronVoxelLinearIndex<QI>, value: QD) -> Option<QD> {
        let slot = self.get_by_linear_index_mut(index)?;
        Some(core::mem::replace(slot, value))
    }

    /// Gets a voxel by X/Y/Z coordinate.
    pub fn get_by_coordinate(&self, coordinate: NeuronVoxelCoordinate<QI>) -> Option<&QD> {
        if !self.contains_coordinate(&coordinate) {
            return None;
        }
        let index = self.dimensions.coordinate_to_linear_index_unchecked(coordinate);
        self.get_by_linear_index(index)
    }

    /// Mutably gets a voxel by X/Y/Z coordinate.
    pub fn get_by_coordinate_mut(&mut self, coordinate: NeuronVoxelCoordinate<QI>) -> Option<&mut QD> {
        if !self.contains_coordinate(&coordinate) {
            return None;
        }
        let index = self.dimensions.coordinate_to_linear_index_unchecked(coordinate);
        self.get_by_linear_index_mut(index)
    }

    /// Sets a voxel by X/Y/Z coordinate, returning the old value when in bounds.
    pub fn set_by_coordinate(&mut self, coordinate: NeuronVoxelCoordinate<QI>, value: QD) -> Option<QD> {
        let slot = self.get_by_coordinate_mut(coordinate)?;
        Some(core::mem::replace(slot, value))
    }

    /// Iterates in xyz order (x-fastest, then y, then z), yielding coordinate/value pairs.
    pub fn iter_xyz(&self) -> impl Iterator<Item = (NeuronVoxelCoordinate<QI>, &QD)> + '_ {
        self.voxels.iter().enumerate().map(move |(i, value)| {
            let linear_index = NeuronVoxelLinearIndex::new(QI::quant_from_usize_unchecked(i));
            (
                self.dimensions
                    .linear_index_to_coordinate_unchecked(linear_index),
                value,
            )
        })
    }

    /// Mutable xyz-order iterator (x-fastest, then y, then z), yielding coordinate/value pairs.
    pub fn iter_xyz_mut(&mut self) -> impl Iterator<Item = (NeuronVoxelCoordinate<QI>, &mut QD)> + '_ {
        let dimensions = self.dimensions;
        self.voxels.iter_mut().enumerate().map(move |(i, value)| {
            let linear_index = NeuronVoxelLinearIndex::new(QI::quant_from_usize_unchecked(i));
            (
                dimensions.linear_index_to_coordinate_unchecked(linear_index),
                value,
            )
        })
    }
}