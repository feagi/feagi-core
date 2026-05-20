use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronMembranePotential};
use feagi_structures::quantization::{CorticalAreaNeuronQuantization, NPUGlobalQuantization};
use feagi_structures::quantization::quantizable_collections::QuantizableCollectionBaseTrait;
use crate::external_models_make_me_my_own_crate::shared_traits_and_structs::NeuronModelParametersTrait;

pub trait NeuronDataBaseTrait<NPUQ: NPUGlobalQuantization, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
: QuantizableCollectionBaseTrait<
    NPUQ::NeuronIndexCountQuant,
    LinearNeuronIndexCount<NPUQ::NeuronIndexCountQuant>,
    (NeuronMembranePotential<CANQ::NeuronDecimalQuant>, NMP)
>
{

}