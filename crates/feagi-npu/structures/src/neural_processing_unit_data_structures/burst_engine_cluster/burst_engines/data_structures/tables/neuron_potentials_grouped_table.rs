use core::marker::PhantomData;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_neuron::NPUNeuronMembranePotential;


pub struct NPUNeuronPotentialsGroupedTableCPU<FGQ: FeagiGlobalQuantization> {
    // TODO other quants
    pub float_32: Vec<NPUNeuronMembranePotential<<CorticalPotentialQuantizationFloat32 as CorticalPotentialQuantization>::NeuronPotentialQuant>>,

    _p: PhantomData<FGQ>,
}











