use serde::{Deserialize, Serialize};
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutChangePolicy, NeuronLayoutRestrictionError, NeuronLayoutRestrictor, NeuronLayoutSparsity, NeuronLayoutTag};
use crate::quantization_levels::FeagiIndexQuantization;

/// General linear layout that can grow
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutSingleSteppedDynamic<FIQ: FeagiIndexQuantization> {
    neuron_count: CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>
}

impl<
    FIQ: FeagiIndexQuantization,
> NeuronLayout<FIQ>
for NeuronLayoutSingleSteppedDynamic<FIQ> {
    
    const CHANGE_POLICY: NeuronLayoutChangePolicy = NeuronLayoutChangePolicy::GrowthBySingleNeuron;
    const SPARSITY: NeuronLayoutSparsity = NeuronLayoutSparsity::Sparse;
    type CorticalLevelContext = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;
    type NeuronLevelContext = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    // yes these are trivial lol
    
    fn new(init_data: Self::CorticalLevelContext) -> Self {
        Self {neuron_count: init_data}
    }

    fn get_number_neurons_to_allocate_for(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_neuron_layout_context(&self, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::NeuronLevelContext {
        neuron_index.clone() 
    }
}



/// Neuron Layout Type of SingleSteppedDynamic
pub struct NeuronLayoutTagSingleSteppedDynamic;

impl NeuronLayoutTag for NeuronLayoutTagSingleSteppedDynamic {
    type Layout<FIQ: FeagiIndexQuantization> = NeuronLayoutSingleSteppedDynamic<FIQ>;
}


//region Restrictions

/// No restrictions for the single stepped dynamic neuron layout. Its already formless
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutRestrictorSingleSteppedDynamicUnbound<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronLayoutRestrictorSingleSteppedDynamicUnbound<FIQ> {
    pub fn new() -> Self { Self(core::marker::PhantomData) }
}

impl<FIQ: FeagiIndexQuantization> NeuronLayoutRestrictor<FIQ>
for NeuronLayoutRestrictorSingleSteppedDynamicUnbound<FIQ>
{
    type CorticalLevelContext = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;

    fn validate_cortical_context(&self, _context: &Self::CorticalLevelContext) -> Result<(), NeuronLayoutRestrictionError> {
        Ok(()) // nothing to check
    }
}

