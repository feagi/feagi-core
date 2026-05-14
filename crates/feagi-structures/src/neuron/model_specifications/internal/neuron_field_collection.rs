use crate::quantization_level::CorticalAreaNeuronQuantization;

pub trait BaseNeuronFieldCollection<CANQ: CorticalAreaNeuronQuantization, DataType> // TODO implement bounds for data type
{
    fn is_sorted(&self) -> bool;
}
