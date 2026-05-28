use core::ops::Range;
use feagi_structures_quantization::{define_quantized_decimal_wrapper_cpu, define_quantized_index_count_wrapper_cpu};
use feagi_structures_quantization::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use feagi_structures_quantization::quantizable_base::decimal::{QuantizedDecimalTrait, QuantizedDecimalWrapperTrait};
use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronMembranePotential};



/// Neuron Voxel Membrane potential -> The potential across a neuron voxel
define_quantized_decimal_wrapper_cpu!(pub struct NeuronVoxelMembranePotential);

impl<QuantDeci: QuantizedDecimalTrait> NeuronVoxelMembranePotential<QuantDeci> {
    pub fn new_from_potential_slice_sum(slice: &[NeuronMembranePotential<QuantDeci>]) -> NeuronVoxelMembranePotential<QuantDeci> {
        slice.iter()
            .fold(
                QuantDeci::QUANT_ZERO,
                |v, &n|
                    v.0.add(n)
            )
    }

    pub fn new_from_potential_slice_average(slice: &[NeuronMembranePotential<QuantDeci>]) -> NeuronVoxelMembranePotential<QuantDeci> {
        NeuronVoxelMembranePotential::wrap_quant(QuantDeci::from_average_of_slice(&slice))
    }

    // TODO MAX function

    // TODO enum based selector

    // TODO enum based in place over an iterator / rayon
}



define_quantized_index_count_wrapper_cpu!(pub struct NeuronVoxelDensity);

impl<QuantLinear: QuantizedIndexCountTrait>  NeuronVoxelDensity<QuantLinear> {
    fn is_single_neuron(&self) -> bool {
        *self.quant_ref() == QuantLinear::QUANT_ONE
    }

    fn is_multi_neuron(&self) -> bool {
        *self.quant_ref() != QuantLinear::QUANT_ONE
    }

    fn is_invalid(&self) -> bool {
        *self.quant_ref() == QuantLinear::QUANT_ZERO
    }
}


define_quantized_index_count_wrapper_cpu!(pub struct VoxelIndexCount);

impl<QuantLinear: QuantizedIndexCountTrait> VoxelIndexCount<QuantLinear> {
    pub fn calculate_linear_index_range(&self,
                                        density: NeuronVoxelDensity<QuantLinear>)
                                        -> Range<LinearNeuronIndexCount<QuantLinear>>
    {
        let start = self / density;
        start..(start + density)
    }
}




define_quantized_index_count_wrapper_cpu!(pub struct VoxelAxisIndex);



