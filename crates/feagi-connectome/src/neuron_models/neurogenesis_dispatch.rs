//! The neurogenesis of a cortical area at any member of the `NeuronModel` family, and its dispatch to code
//! needing the concrete neuron model

use crate::neuron_models::family::{NMContainer, NMEnum};
use core::marker::PhantomData;
pub use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGeneratorOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

/// Holds the [`NeurogenesisFromStorageOrByGeneratorOf`] of a cortical area
pub struct NeurogenesisFromStorageOrByGeneratorContainer<FIQ>(PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NeurogenesisFromStorageOrByGeneratorContainer<FIQ> {
    type Of<M: NeuronModel> = NeurogenesisFromStorageOrByGeneratorOf<FIQ, M>;
}

/// The neurogenesis of a cortical area, at whichever member of the `NeuronModel` family it uses
pub type NeurogenesisFromStorageOrByGeneratorEnum<FIQ> = NMEnum<NeurogenesisFromStorageOrByGeneratorContainer<FIQ>>;

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaNeurogenesisDispatch<FIQ>
for NeurogenesisFromStorageOrByGeneratorEnum<FIQ> {
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
    type CorticalAreaNeurogenesisDispatch = NeurogenesisFromStorageOrByGeneratorEnum<FIQ>;
}
