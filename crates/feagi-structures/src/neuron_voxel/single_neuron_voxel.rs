//! In Neuron Voxels where there is only 1 neuron per voxel (density of 1). Most common case
//! Since we have 1 less dimension to iterate, using this can make some computations faster

use feagi_structures_quantization::define_unsigned_spatial_3d_cpu_wrappers;
use feagi_structures_quantization::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use feagi_structures_quantization::quantizable_spatial::unsigned_spatial_3d::SpatialDimension3DTrait;
use crate::neuron::LinearNeuronIndexCount;
use crate::neuron_voxel::neuron_voxel_common::{VoxelAxisIndex, VoxelIndexCount};

define_unsigned_spatial_3d_cpu_wrappers!(
    pub struct SingleNeuronVoxelCoordinate,
    pub struct SingleNeuronVoxelDimensions,
    VoxelIndexCount<QuantIndex>,
    VoxelAxisIndex<QuantIndex>,
    VoxelAxisIndex<QuantIndex>,
    VoxelAxisIndex<QuantIndex>
);

impl<QuantLinear: QuantizedIndexCountTrait> SingleNeuronVoxelDimensions<QuantLinear, SingleNeuronVoxelCoordinate<QuantLinear>> {

    pub fn new_voxel_dims_unchecked(x_dim: VoxelAxisIndex<QuantLinear>, y_dim: VoxelAxisIndex<QuantLinear>, z_dim: VoxelAxisIndex<QuantLinear>) -> Self {
        
    }
    
    pub fn get_max_voxel_index(&self) -> VoxelIndexCount<QuantLinear> {
        self.get_max_linear_index()
    }
    
    pub fn get_max_neuron_index(&self) -> LinearNeuronIndexCount<QuantLinear> {
        LinearNeuronIndexCount::wrap_quant(self.get_max_linear_index().quant_ref())
    }
    
    // TODO neuron coord <-> index conversion

}