use crate::define_ref_access_trait_methods;
use crate::neuron::individual_neuron_structs::{IndividualNeuronIndexCount, IndividualNeuronMembranePotential};
use crate::neuron::neuron_model::neuron_collection_type::NeuronCollectionType;
use crate::quantization_level::CorticalAreaNeuronQuantization;


//region Neuron
/// Defines the properties of a single neuron of a given Neuron Model
pub trait IndividualNeuronModelBaseTrait<CANQ: CorticalAreaNeuronQuantization> {

    //type NeuronFlagType: NeuronFlagAliveTrait;
    define_ref_access_trait_methods!(membrane_potential, IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>);

    // TODO enumerated iterators?
}



//endregion

//region Collection

pub trait NeuronModelContainerBaseTrait<CANQ: CorticalAreaNeuronQuantization> {
    type IndividualNeuronModelType: IndividualNeuronModelBaseTrait<CANQ>;
    const NEURON_COLLECTION_TYPE: NeuronCollectionType;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_value_max_index(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_number_contained_neuron_values(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;
}


pub trait NeuronModelCollectionSparseBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelContainerBaseTrait<CANQ>
{
    // TODO macro for enumerated iterator generation

}

pub trait NeuronModelDenseCollectionBaseTrait<CANQ: CorticalAreaNeuronQuantization>:
NeuronModelContainerBaseTrait<CANQ>
{
    define_ref_access_trait_methods!(membrane_potentials, [IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>]);
}

//endregion

//region Cortical Area Level

pub trait NeuronModelCorticalAreaDataBaseTrait<CANQ: CorticalAreaNeuronQuantization> {
    type CorticalFlagType: CorticalFlagAliveTrait;
}




//endregion