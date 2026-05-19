use feagi_structures::base_feagi_types::quantizable_types::QuantizableUIntType;
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount, NeuronDensityTrait};
use crate::dynamics::neuron::shared::collections::multi_neuron_density::shared::NeuronModelCollectionMultiNeuronLinearTrait;
use crate::dynamics::neuron::shared::collections::single_neuron_density::shared::NeuronModelCollectionSingleNeuronLinearTrait;
use crate::dynamics::neuron::shared::neurons::NeuronModelParametersTrait;


pub enum NeuronLinearResizeMethod {
    TruncateOrExpandDefault,
    ResetAllDefault
}

/// Optional Trait that allows linear resizing
pub trait NeuronModelCollectionSingleNeuronLinearResizableTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionSingleNeuronLinearTrait<CANQ, NMP>
{
    fn resize_single_linear_neuron_collection(&mut self,
                                              new_total_neuron_count: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
                                              resize_method: NeuronLinearResizeMethod) -> Result<(), FeagiNeuronError>;
}



/// Optional Trait that allows linear resizing
pub trait NeuronModelCollectionMultiNeuronLinearResizableTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>:
NeuronModelCollectionMultiNeuronLinearTrait<CANQ, NMP, ND, NeuronSetIndexTrait>
{
    fn resize_multi_linear_neuron_collection(&mut self,
                                             new_total_neuron_set_count: NeuronSetIndexTrait,
                                             new_density: ND,
                                             resize_method: NeuronLinearResizeMethod) -> Result<(), FeagiNeuronError>;
}
