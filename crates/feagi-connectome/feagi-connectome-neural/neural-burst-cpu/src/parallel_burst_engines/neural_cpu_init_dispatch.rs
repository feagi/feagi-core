use crate::parallel_burst_engines::neural_cpu_parallel_dynamics::NeuronCPUParallelDynamics;
use neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use neural_basis::neuron_model::NeuronModel;
use neural_basis::quantization_levels::FeagiIndexQuantization;

/// How a cortical area of neuron model `NM` gets spawned: either from stored neuron data, or by
/// that model's own generator
pub type CpuCorticalAreaNeurogenesis<FIQ, NM> =
    NeurogenesisFromStorageOrByGenerator<FIQ, NM, <NM as NeuronModel>::NeurogenesisGenerator<FIQ>>;

/// Something that wants a cortical area's neurogenesis with its concrete neuron model, along with
/// the CPU dynamics of that model
pub trait CpuCorticalAreaInitVisitor<FIQ: FeagiIndexQuantization> {
    type Output;
    fn visit<NM, D>(self, neurogenesis: CpuCorticalAreaNeurogenesis<FIQ, NM>) -> Self::Output
    where
        NM: NeuronModel,
        D: NeuronCPUParallelDynamics<FIQ, NM>;
}

/// Something that holds a cortical area init and can give it, with its concrete neuron model and
/// the CPU dynamics of that model, to a visitor
pub trait CpuCorticalAreaInitDispatch<FIQ: FeagiIndexQuantization>: Send {
    fn dispatch_cpu<V: CpuCorticalAreaInitVisitor<FIQ>>(self, visitor: V) -> V::Output;
}
