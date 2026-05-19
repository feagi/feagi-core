use feagi_structures::CorticalAreaNeuronQuantization;
use crate::dynamics::neuron::shared::neurons::NeuronModelParametersTrait;

pub trait NeuronCollectionDimensionalShared<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    fn get_voxel_dimensions(&self) -> Dimenbsions;

    

}