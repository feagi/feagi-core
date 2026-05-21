use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronMembranePotential};
use feagi_structures::quantization::{CorticalAreaNeuronQuantization, NPUGlobalQuantization};
use feagi_structures::quantization::quantizable_collections::QuantizableCollectionBaseTrait;
use feagi_npu_neuron_models::::NeuronModelParametersTrait;

pub trait NeuronDataBaseTrait<NPUQ: NPUGlobalQuantization, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
: QuantizableCollectionBaseTrait<
    NPUQ::NeuronIndexCountQuant,
    LinearNeuronIndexCount<NPUQ::NeuronIndexCountQuant>,
    (NeuronMembranePotential<CANQ::NeuronDecimalQuant>, NMP)
>
{

}