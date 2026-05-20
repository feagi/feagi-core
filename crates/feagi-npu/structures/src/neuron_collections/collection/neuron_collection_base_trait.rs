use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronMembranePotential};
use feagi_structures::quantization::{CorticalAreaNeuronQuantization, NPUGlobalQuantization};
use feagi_structures::quantization::quantizable_collections::QuantizableCollectionBaseTrait;


pub trait NeuronDataBaseTrait<NPUQ: NPUGlobalQuantization, CANQ: CorticalAreaNeuronQuantization>
: QuantizableCollectionBaseTrait<
    NPUQ::NeuronIndexCountQuant,
    LinearNeuronIndexCount<NPUQ::NeuronIndexCountQuant>,
    (NeuronMembranePotential<CANQ::NeuronDecimalQuant>, NeuronModelParametersTrait<CANQ>)
>
{

}