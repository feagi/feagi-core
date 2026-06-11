use core::marker::PhantomData;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::calculate_struct_padding::calculate_byte_alignment_padding;
use crate::neural_processing_unit_data_structures::cpu_wrappers::{ NPUWrappedNeuronMembranePotential};

/// Holds membrane potential data, in subgroups by quantization, of all neurons in the
/// connectome. Globally, is ordered from f32, f8, f16, f64, u8
pub trait NPUNeuronMembranePotentialsMPGroupedTable<FGQ: FeagiGlobalQuantization> {}


// TODO other quants!
type F32Quant = <CorticalPotentialQuantizationFloat32 as CorticalPotentialQuantization>::NeuronPotentialQuant;

/// Neuron Potentials Grouped by MP Quantization. This struct is used for holding the neuron
/// membrane potentials and for the FCL. Make sure to use NPUWrappedNeuronMPQuantIndex indexing
#[repr(C)]
pub struct NPUNeuronMembranePotentialsMPGroupedTableCPU<FGQ: FeagiGlobalQuantization>
{
    pub float_32: Vec<NPUWrappedNeuronMembranePotential<F32Quant>>,
    _padding_1: [u8; calculate_byte_alignment_padding(size_of::<Vec<u8>>())], // data type irrelevant, have one per vector
    // TODO f8
    // TODO f16
    // TODO f64
    // TODO u8?
    _p: PhantomData<FGQ>,

}

impl<FGQ: FeagiGlobalQuantization> NPUNeuronMembranePotentialsMPGroupedTable<FGQ> for NPUNeuronMembranePotentialsMPGroupedTableCPU<FGQ> {}








