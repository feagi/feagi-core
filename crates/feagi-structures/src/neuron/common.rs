use core::ops::Range;
use crate::base_feagi_types::quantizable_types::{QuantizableUIntType};
use crate::neuron::feagi_neuron_error::FeagiNeuronError;
use crate::neuron::neuron_density::NeuronDensityTrait;


/// Linear indexing and counting of neurons
crate::define_quantizable_uint_type_family!(LinearNeuronIndexCount);

/// Neuron Membrane potential
crate::define_quantizable_value_type_family!(NeuronMembranePotential);


//region Number Neurons Per Voxel

/// Defines the number of neurons within a voxel
pub struct NeuronVoxelDensity(u8);

impl NeuronVoxelDensity {
    pub fn new(value: u8) -> Result<NeuronVoxelDensity, FeagiNeuronError> {
        if value == 0 {
            return Err(FeagiNeuronError::InvalidVoxelDensity {
                context: "Neuron Density cannot be zero!"
            })
        }
        Ok(NeuronVoxelDensity(value))
    }
    
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl NeuronDensityTrait for NeuronVoxelDensity {
    fn number_of_neurons_per_unit(&self) -> u8 {
        self.0
    }
}

//endregion


/// Neuron Voxel Membrane potential
crate::define_quantizable_value_type_family!(NeuronVoxelMembranePotential);

//region Neuron Voxel Index and Count
crate::define_quantizable_uint_type_family!(VoxelIndexCount);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> VoxelIndexCount<VoxelIndexCountCoordQuant> {
    pub fn calculate_linear_index_range(&self,
                                        density: NeuronVoxelDensity)
                                        -> Range<LinearNeuronIndexCount<VoxelIndexCountCoordQuant>>
    {
        let start = self.to_usize() / density.as_usize();
        LinearNeuronIndexCount::from_usize(start)..LinearNeuronIndexCount::from_usize(start + density.as_usize())
    }
}
//endregion


//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(VoxelCoordinate);

//endregion


//region Neuron Voxel Dimensions


crate::define_dimension_3d_type_family!(VoxelDimensions, VoxelCoordinate);

impl<VoxelIndexCountCoordQuant: QuantizableUIntType> VoxelDimensions<VoxelIndexCountCoordQuant> {

    pub fn get_number_voxels(&self) -> VoxelIndexCount<VoxelIndexCountCoordQuant> {
        VoxelIndexCount::from_usize(self.number_elements())
    }

    pub fn get_number_neurons(&self, density: &NeuronVoxelDensity) -> LinearNeuronIndexCount<VoxelIndexCountCoordQuant> {
        LinearNeuronIndexCount::from_usize(self.number_elements() * density.as_usize())
    }

    // TODO remove to_usize conversions
    /// Linear voxel index with **x varying fastest**: `index = x + y·dx + z·dx·dy`.
    #[inline(always)]
    pub fn voxel_index_to_voxel_coordinate(
        &self,
        voxel_index: VoxelIndexCount<VoxelIndexCountCoordQuant>,
    ) -> VoxelCoordinate<VoxelIndexCountCoordQuant> {
        let i = QuantizableUIntType::to_usize(voxel_index);
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let plane = dx * dy;
        let z = i / plane;
        let rem = i % plane;
        let y = rem / dx;
        let x = rem % dx;
        VoxelCoordinate::new(
            VoxelIndexCountCoordQuant::from_usize(x),
            VoxelIndexCountCoordQuant::from_usize(y),
            VoxelIndexCountCoordQuant::from_usize(z),
        )
    }

    /// Inverse of [`Self::linear_index_to_coordinate`].
    #[inline(always)]
    pub fn voxel_coordinate_to_voxel_index(
        &self,
        coordinate: VoxelCoordinate<VoxelIndexCountCoordQuant>,
    ) -> VoxelIndexCount<VoxelIndexCountCoordQuant> {
        let dx = self.x.get().to_usize();
        let dy = self.y.get().to_usize();
        let x = coordinate.x.to_usize();
        let y = coordinate.y.to_usize();
        let z = coordinate.z.to_usize();
        let i = x + y * dx + z * dx * dy;
        VoxelIndexCount::from_usize(i)
    }

    // TODO iterators
}

//endregion

