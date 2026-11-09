//! The neurogenesis of a cortical area at any member of the `NeuronModel` family, and its dispatch to code
//! needing the concrete neuron model

use crate::neuron_models::family::{NMContainer, NMEnum};
use core::marker::PhantomData;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

/// How a cortical area of model `NM` gets spawned: either from stored neuron data, or by that
/// model's own generator
pub type CorticalAreaNeurogenesis<FIQ, NM> =
    NeurogenesisFromStorageOrByGenerator<FIQ, NM, <NM as NeuronModel>::NeurogenesisGenerator<FIQ>>;

/// Holds the [`CorticalAreaNeurogenesis`] of a cortical area
pub struct CorticalAreaNeurogenesisContainer<FIQ>(PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for CorticalAreaNeurogenesisContainer<FIQ> {
    type Of<M: NeuronModel> = CorticalAreaNeurogenesis<FIQ, M>;
}

/// The neurogenesis of a cortical area, at whichever member of the `NeuronModel` family it uses
pub type CorticalAreaNeurogenesisEnum<FIQ> = NMEnum<CorticalAreaNeurogenesisContainer<FIQ>>;

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaNeurogenesisDispatch<FIQ>
for CorticalAreaNeurogenesisEnum<FIQ> {
    fn dispatch_cpu<V: feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaNeurogenesisVisitor<FIQ>>(
        self,
        visitor: V,
    ) -> V::Output {
        with_nm!(self, <NM, Model> |neurogenesis| {
            visitor.visit::<NM, Model::BurstEngineDynamics<FIQ, NM>>(neurogenesis)
        })
    }
}

/// The dispatch types of this connectome, for the connectome edit steps
#[cfg(feature = "std")]
pub struct ConnectomeDispatch;

#[cfg(feature = "std")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::neural_basis::connectome_editing::ConnectomeVisitorDispatchWrapper<FIQ>
for ConnectomeDispatch {
    type CorticalAreaNeurogenesisDispatch = CorticalAreaNeurogenesisEnum<FIQ>;
}
