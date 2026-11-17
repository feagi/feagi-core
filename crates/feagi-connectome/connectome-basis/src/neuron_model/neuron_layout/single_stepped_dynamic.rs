use serde::{Deserialize, Serialize};
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutChangePolicy, NeuronLayoutError, NeuronLayoutRestrictor, NeuronLayoutSparsity, NeuronLayoutTag};
use crate::quantization_levels::FeagiIndexQuantization;

/// General linear layout that can grow up to one neuron per burst
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutSingleSteppedDynamic<FIQ: FeagiIndexQuantization> {
    neuron_count: CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization, > NeuronLayout<FIQ> for NeuronLayoutSingleSteppedDynamic<FIQ> {
    const CHANGE_POLICY: NeuronLayoutChangePolicy = NeuronLayoutChangePolicy::GrowthBySingleNeuron;
    const SPARSITY: NeuronLayoutSparsity = NeuronLayoutSparsity::Sparse;
    type CorticalLevelContext = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;
    type NeuronLevelContext = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;
    
    // yes these are trivial lol

    fn get_cortical_layout_context(&self) -> &Self::CorticalLevelContext {
        &self.neuron_count
    }

    fn get_number_neurons_to_allocate_for(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_count
    }

    fn get_neuron_layout_context(
        &self,
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>)
        -> Result<Self::NeuronLevelContext, NeuronLayoutError> {
        Ok(neuron_index.clone())
    }
}


/// No restrictions for the single stepped dynamic neuron layout. Its already formless
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutRestrictorSingleSteppedDynamicUnbound<FIQ: FeagiIndexQuantization>(core::marker::PhantomData<FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronLayoutRestrictor<FIQ> for NeuronLayoutRestrictorSingleSteppedDynamicUnbound<FIQ>
{
    type NeuronLayout = NeuronLayoutSingleSteppedDynamic<FIQ>;

    fn validate_cortical_context(context: &<Self::NeuronLayout as NeuronLayout<FIQ>>::CorticalLevelContext) -> Result<(), NeuronLayoutError> {
        if context.is_zero() {
            Err(NeuronLayoutError::EmptyNeuronLayout)
        } else {
            Ok(())
        }

    }
    
    fn validate_and_create_neuron_layout(context: <Self::NeuronLayout as NeuronLayout<FIQ>>::CorticalLevelContext) -> Result<Self::NeuronLayout, NeuronLayoutError> {
        Self::validate_cortical_context(&context)?;
        Ok(NeuronLayoutSingleSteppedDynamic {neuron_count: context})
    }
}


/// Neuron Layout Type of SingleSteppedDynamic
#[derive(Copy, Clone, Debug)]
pub struct NeuronLayoutTagSingleSteppedDynamic;

impl NeuronLayoutTag for NeuronLayoutTagSingleSteppedDynamic {
    type Layout<FIQ: FeagiIndexQuantization> = NeuronLayoutSingleSteppedDynamic<FIQ>;
}
