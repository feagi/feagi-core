use crate::parallel_burst_engines::cortical_cpu_parallel_dynamics::CorticalCPUParallelDynamics;
use neural_basis::neuron_model::interfacing::cortical_area_spec::CorticalAreaSpec;
use neural_basis::neuron_model::NeuronModel;
use neural_basis::quantization_levels::FeagiIndexQuantization;

/// Something that wants a cortical area spec with its concrete neuron model, along with the CPU
/// dynamics of that model
pub trait CpuCorticalAreaSpecVisitor<FIQ: FeagiIndexQuantization> {
    type Output;
    fn visit<NM, D>(self, spec: CorticalAreaSpec<FIQ, NM>) -> Self::Output
    where
        NM: NeuronModel,
        D: CorticalCPUParallelDynamics<FIQ, NM>;
}

/// Something that holds a cortical area spec and can give it, with its concrete neuron model and
/// the CPU dynamics of that model, to a visitor
pub trait CpuCorticalAreaSpecDispatch<FIQ: FeagiIndexQuantization>: Send {
    fn dispatch_cpu<V: CpuCorticalAreaSpecVisitor<FIQ>>(self, visitor: V) -> V::Output;
}
