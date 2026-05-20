use core::ops::Range;
use feagi_structures_quantization::{define_quantized_index_count_wrapper_cpu, define_quantized_decimal_wrapper_cpu, define_unsigned_spatial_3d_cpu_wrappers};
use feagi_structures_quantization::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::neuron::{LinearNeuronIndexCount, NeuronDensityTrait, NeuronMembranePotential};
use crate::neuron_voxels::FeagiNeuronVoxelError;



/// Describes what method a voxel's potential is calculated if it has multiple inner neurons
pub enum NeuronVoxelMultiPotentialCalculationMethod {
    Sum,
    Average,
    Max
}


/// Neuron Voxel Membrane potential -> The potential across a neuron voxel
define_quantized_decimal_wrapper_cpu!(pub struct NeuronVoxelMembranePotential);

impl<QuantDeci: QuantizedIndexCountTrait> NeuronVoxelMembranePotential<QuantDeci> {
    pub fn new_from_potential_slice_sum(slice: &[NeuronMembranePotential<QuantDeci>]) -> NeuronVoxelMembranePotential<QuantDeci> {
        slice.iter()
            .fold(
                QuantDeci::QUANT_ZERO,
                |v, &n|
                    v.0.add(n)
            )
    }

    pub fn new_from_potential_slice_average(slice: &[NeuronMembranePotential<QuantDeci>]) -> NeuronVoxelMembranePotential<QuantDeci> {
        slice.iter()
            .fold(
                QuantDeci::QUANT_ZERO,
                |v, &n|
                    v.0.add(n)
            )
        // TODO make better average, maybe built in?
    }
}



define_quantized_index_count_wrapper_cpu!(pub struct NeuronVoxelDensity);

impl<QuantLinear: QuantizedIndexCountTrait> NeuronDensityTrait<QuantLinear> for NeuronVoxelDensity<QuantLinear> {
    fn number_of_neurons_per_unit(&self) -> QuantLinear {
        self.0
    }
}


define_quantized_index_count_wrapper_cpu!(pub struct VoxelIndexCount);
define_quantized_index_count_wrapper_cpu!(pub struct VoxelAxisIndex);

impl<QuantLinear: QuantizedIndexCountTrait> VoxelIndexCount<QuantLinear> {
    pub fn calculate_linear_index_range(&self,
                                        density: NeuronVoxelDensity<QuantLinear>)
                                        -> Range<LinearNeuronIndexCount<QuantLinear>>
    {
        let start = self / density;
        start..(start + density)
    }
}


define_unsigned_spatial_3d_cpu_wrappers!(
    pub struct VoxelCoordinate,
    pub struct VoxelDimensions,
    VoxelIndexCount<QuantIndex>,
    VoxelAxisIndex<QuantIndex>,
    VoxelAxisIndex<QuantIndex>,
    VoxelAxisIndex<QuantIndex>
);



impl<QuantLinear: QuantizedIndexCountTrait> VoxelDimensions<QuantLinear, VoxelCoordinate<QuantLinear>> {

    pub fn get_number_voxels(&self) -> VoxelIndexCount<QuantLinear> {
        VoxelIndexCount::wrap_quant(QuantLinear::from_usize(self.number_elements()))
    }

    pub fn get_number_neurons(&self, density: &NeuronVoxelDensity<QuantLinear>) -> LinearNeuronIndexCount<QuantLinear> {
        self.get_number_voxels().0 * density.0
    }

    // TODO neuron coord <-> index conversion

}

// TODO Dense Voxel 
