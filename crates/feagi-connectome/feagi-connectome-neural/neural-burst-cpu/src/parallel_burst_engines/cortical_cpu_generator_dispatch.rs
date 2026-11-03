use neural_basis::neuron_model::interfacing::generators::{CorticalAreaGenerator, CorticalAreaGeneratorDispatch};
use neural_basis::neuron_model::neuron_model::NeuronModel;
use neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::parallel_burst_engines::cortical_cpu_parallel_dynamics::CorticalCPUParallelDynamics;

/// Something that wants a generator with its concrete types, along with the CPU dynamics of its
/// neuron model
pub trait CpuCorticalAreaGeneratorVisitor<FIQ: FeagiIndexQuantization> {
    type Output;
    fn visit<NM, CAG, D>(self, generator: CAG) -> Self::Output
    where
        NM: NeuronModel,
        CAG: CorticalAreaGenerator<FIQ, NM>,
        D: CorticalCPUParallelDynamics<FIQ, NM>;
}

/// Something that holds a generator and can give it, with its real types and the CPU dynamics of
/// its neuron model, to a visitor
pub trait CpuCorticalAreaGeneratorDispatch<FIQ: FeagiIndexQuantization>: CorticalAreaGeneratorDispatch<FIQ> {
    fn dispatch_cpu<V: CpuCorticalAreaGeneratorVisitor<FIQ>>(self, visitor: V) -> V::Output;
}
