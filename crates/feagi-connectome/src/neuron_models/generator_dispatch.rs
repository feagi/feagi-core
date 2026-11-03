//! The generator of a cortical area at any member of the `NeuronModel` family, and its dispatch to
//! code needing the concrete neuron model and generator types

use super::neuron_model;
use super::neuron_model::{NeuronModel, NeuronModelContainer, NeuronModelEnum};
use core::marker::PhantomData;
use feagi_connectome_neural::neural_basis::neuron_model::interfacing::generators::{
    CorticalAreaGeneratorDispatch, CorticalAreaGeneratorFromDefaults, CorticalAreaGeneratorVisitor,
};
use feagi_connectome_neural::neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Holds the [`CorticalAreaGeneratorFromDefaults`] of a cortical area
pub struct FromDefaultsGenerator<FIQ>(PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronModelContainer for FromDefaultsGenerator<FIQ> {
    type Of<M: NeuronModel> = CorticalAreaGeneratorFromDefaults<FIQ, M>;
}

/// The generator of a cortical area, at whichever member of the `NeuronModel` family it uses
pub type CorticalAreaGeneratorEnum<FIQ> = NeuronModelEnum<FromDefaultsGenerator<FIQ>>;

impl<FIQ: FeagiIndexQuantization> CorticalAreaGeneratorDispatch<FIQ> for CorticalAreaGeneratorEnum<FIQ> {
    fn dispatch<V: CorticalAreaGeneratorVisitor<FIQ>>(self, visitor: V) -> V::Output {
        with_neuron_model!(self, <NM> |generator| visitor.visit::<NM, _>(generator))
    }
}

#[cfg(feature = "burst-engine-cpu")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaGeneratorDispatch<FIQ>
for CorticalAreaGeneratorEnum<FIQ> {
    fn dispatch_cpu<V: feagi_connectome_neural::burst_engine_cpu::CpuCorticalAreaGeneratorVisitor<FIQ>>(
        self,
        visitor: V,
    ) -> V::Output {
        with_neuron_model!(self, <NM, Model> |generator| {
            visitor.visit::<NM, _, Model::BurstEngineDynamics<FIQ, NM>>(generator)
        })
    }
}

/// The dispatch types of this connectome, for the connectome edit steps
#[cfg(feature = "std")]
pub struct ConnectomeDispatch;

#[cfg(feature = "std")]
impl<FIQ: FeagiIndexQuantization> feagi_connectome_neural::neural_basis::connectome_editing::ConnectomeVisitorDispatchWrapper<FIQ>
for ConnectomeDispatch {
    type CorticalAreaGeneratorDispatch = CorticalAreaGeneratorEnum<FIQ>;
}
