use core::ops::Range;
use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType, QuantizableUIntType, QuantizableValueType};
use crate::base_feagi_types::quantizable_types::spatial::QuantizableUInt3DDimensionType;
use crate::neuron::{LinearNeuronIndexCount, NeuronDensityTrait, NeuronMembranePotential};
use crate::neuron_voxels::FeagiNeuronVoxelError;


//region Neuron Voxel Density

/// Defines the number of neurons within a voxel
pub struct NeuronVoxelDensity(u8);

impl NeuronVoxelDensity {
    pub fn new(value: u8) -> Result<NeuronVoxelDensity, FeagiNeuronVoxelError> {
        if value == 0 {
            return Err(FeagiNeuronVoxelError::InvalidVoxelDensity {
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

//region Neuron Voxel Membrane Potential

/// Describes what method a voxel's potential is calculated if it has multiple inner neurons
pub enum NeuronVoxelMultiPotentialCalculationMethod {
    Sum,
    Average,
    Max
}

/// Neuron Voxel Membrane potential -> The potential across a neuron voxel
crate::define_quantizable_value_type_family!(NeuronVoxelMembranePotential);

impl<Q: QuantizableValueType> NeuronVoxelMembranePotential<Q>
{
    pub fn new_from_potential_slice_sum(slice: &[NeuronMembranePotential<Q>]) -> NeuronVoxelMembranePotential<Q> {
        slice.iter()
            .fold(
                NeuronVoxelMembranePotential::ZERO,
                |v, &n|
                    v.saturating_add(NeuronVoxelMembranePotential(n.0))
            )
    }

    pub fn new_from_potential_slice_average(slice: &[NeuronMembranePotential<Q>]) -> NeuronVoxelMembranePotential<Q> {
        slice.iter()
            .fold(
                NeuronVoxelMembranePotential::ZERO,
                |v, &n|
                    v.saturating_add(NeuronVoxelMembranePotential(n.0))
            )
    }
    



}



//region Neuron Voxel Index and Count
crate::define_quantizable_uint_type_family!(VoxelIndexCount);

//endregion


//region Neuron Voxel Coordinate

crate::define_unsigned_coordinate_3d_type_family!(VoxelCoordinate);

//endregion


//region Neuron Voxel Dimensions


crate::define_dimension_3d_type_family!(VoxelDimensions, VoxelCoordinate);

//endregion



impl<VoxelIndexCountCoordQuant: QuantizableUIntType> VoxelIndexCount<VoxelIndexCountCoordQuant> {
    pub fn calculate_linear_index_range(&self,
                                        density: NeuronVoxelDensity)
                                        -> Range<LinearNeuronIndexCount<VoxelIndexCountCoordQuant>>
    {
        let start = self.to_usize() / density.as_usize();
        LinearNeuronIndexCount::from_usize(start)..LinearNeuronIndexCount::from_usize(start + density.as_usize())
    }
}

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
