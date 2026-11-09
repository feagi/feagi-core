//! The init of a cortical area at any member of the `NeuronModel` family, and its dispatch to code
//! needing the concrete neuron model

use crate::neuron_models::family::{neuron_model, NeuronModelContainer, NeuronModelEnum};
use core::marker::PhantomData;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

/// How a cortical area of model `NM` gets spawned: either from stored neuron data, or by that
/// model's own generator
pub type CorticalAreaNeurogenesis<FIQ, NM> =
    NeurogenesisFromStorageOrByGenerator<FIQ, NM, <NM as NeuronModel>::NeurogenesisGenerator<FIQ>>;

/// Holds the [`CorticalAreaNeurogenesis`] of a cortical area
pub struct CorticalAreaInitContainer<FIQ>(PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronModelContainer for CorticalAreaInitContainer<FIQ> {
    type Of<M: NeuronModel> = CorticalAreaNeurogenesis<FIQ, M>;
}

/// The init of a cortical area, at whichever member of the `NeuronModel` family it uses
pub type CorticalAreaInitEnum<FIQ> = NeuronModelEnum<CorticalAreaInitContainer<FIQ>>;

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaInitDispatch<FIQ>
for CorticalAreaInitEnum<FIQ> {
    fn dispatch_cpu<V: feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaInitVisitor<FIQ>>(
        self,
        visitor: V,
    ) -> V::Output {
        with_neuron_model!(self, <NM, Model> |init| {
            visitor.visit::<NM, Model::BurstEngineDynamics<FIQ, NM>>(init)
        })
    }
}

/// The dispatch types of this connectome, for the connectome edit steps
#[cfg(feature = "std")]
pub struct ConnectomeDispatch;

#[cfg(feature = "std")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::neural_basis::connectome_editing::ConnectomeVisitorDispatchWrapper<FIQ>
for ConnectomeDispatch {
    type CorticalAreaInitDispatch = CorticalAreaInitEnum<FIQ>;
}
