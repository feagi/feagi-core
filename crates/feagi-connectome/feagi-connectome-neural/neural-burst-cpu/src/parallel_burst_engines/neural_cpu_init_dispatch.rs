use crate::parallel_burst_engines::neural_cpu_parallel_dynamics::NeuronCPUParallelDynamics;
use neural_basis::neuron_model::interfacing::cortical_area_init::CorticalAreaInit;
use neural_basis::neuron_model::NeuronModel;
use neural_basis::quantization_levels::FeagiIndexQuantization;

/// Something that wants a cortical area init with its concrete neuron model, along with the CPU
/// dynamics of that model
pub trait CpuCorticalAreaInitVisitor<FIQ: FeagiIndexQuantization> {
    type Output;
    fn visit<NM, D>(self, init: CorticalAreaInit<FIQ, NM>) -> Self::Output
    where
        NM: NeuronModel,
        D: NeuronCPUParallelDynamics<FIQ, NM>;
}

/// Something that holds a cortical area init and can give it, with its concrete neuron model and
/// the CPU dynamics of that model, to a visitor
pub trait CpuCorticalAreaInitDispatch<FIQ: FeagiIndexQuantization>: Send {
    fn dispatch_cpu<V: CpuCorticalAreaInitVisitor<FIQ>>(self, visitor: V) -> V::Output;
}
