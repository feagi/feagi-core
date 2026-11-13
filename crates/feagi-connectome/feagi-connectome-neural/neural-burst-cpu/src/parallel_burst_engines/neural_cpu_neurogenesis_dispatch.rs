use neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisStorageOrGenerator;
use crate::parallel_burst_engines::neural_cpu_parallel_dynamics::NeuronCPUParallelDynamics;
use neural_basis::neuron_model::NeuronModel;
use neural_basis::quantization_levels::FeagiIndexQuantization;

/// Something that wants a cortical area's neurogenesis with its concrete neuron model, along with
/// the CPU dynamics of that model
pub trait CpuCorticalAreaNeurogenesisVisitor<FIQ: FeagiIndexQuantization> {
    type Output;
    fn visit<NM, D>(self, neurogenesis: ValidatedNeurogenesisStorageOrGenerator<FIQ, NM>) -> Self::Output
    where
        NM: NeuronModel,
        D: NeuronCPUParallelDynamics<FIQ, NM>;
}

/// Something that holds the neurogenesis of a cortical area and can give it, with its concrete neuron model and
/// the CPU dynamics of that model, to a visitor
pub trait CpuCorticalAreaNeurogenesisDispatch<FIQ: FeagiIndexQuantization>: Send {
    fn dispatch_cpu<V: CpuCorticalAreaNeurogenesisVisitor<FIQ>>(self, visitor: V) -> V::Output;
}
