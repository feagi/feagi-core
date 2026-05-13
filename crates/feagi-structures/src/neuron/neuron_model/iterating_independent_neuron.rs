use crate::neuron::neuron_model::dimensional::dimensional_structs::NeuronVoxelIndexCount;
use crate::neuron::neuron_model::neuron_model_base_traits::IndividualNeuronModelBaseTrait;
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// Neuron Voxel iteration item. Since voxels will only show neuron potential, 
/// we will only include that as a value
pub struct IndividualNeuronIterItem<'a, CANQ: CorticalAreaNeuronQuantization, NMI: IndividualNeuronModelBaseTrait<CANQ>>
{
    pub individual_neuron_index: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_model_collection: &'a NMI
}

pub enum NeuronSmartIterator<CANQ, SDI, MDI, ENMI>
where
    CANQ: CorticalAreaNeuronQuantization,
//SDI: SingleDensityIterator,
//MDI: MultipleDensityIterator,
//EDMN: NeuronIterItem<CANQ>
{
    Single(SDI),
    Multiple(MDI),
}

impl<CANQ, SDI, MDI, EDMN> Iterator for NeuronSmartIterator<CANQ, SDI, MDI, EDMN>
where
    CANQ: CorticalAreaNeuronQuantization,
//SDI: SingleDensityIterator<Item = EDMN>,
//MDI: MultipleDensityIterator<Item = EDMN>,
//EDMN: EnumeratedDimensionalModelNeuron<CANQ>
{
    type Item = EDMN;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NeuronSmartIterator::Single(iter) => iter.next(),
            NeuronSmartIterator::Multiple(iter) => iter.next(),
        }
    }
}

impl<CANQ, SDI, MDI, EDMN> NeuronSmartIterator<CANQ, SDI, MDI, EDMN>
where
    CANQ: CorticalAreaNeuronQuantization,
//SDI: SingleDensityIterator<Item = DMN>,
//MDI: MultipleDensityIterator<Item = DMN>,
//EDMN: EnumeratedDimensionalModelNeuron<CANQ>
{

}