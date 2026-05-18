use crate::define_ref_immut_mut_access_concrete_methods;
use crate::quantization_level::CorticalAreaNeuronQuantization;

//region Neuron
pub struct PseudoVoxelModelNeuron<CANQ: CorticalAreaNeuronQuantization> {
    membrane_potential: IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
}

impl<CANQ: CorticalAreaNeuronQuantization> IndividualNeuronModelBaseTrait<CANQ> for PseudoVoxelModelNeuron<CANQ> {
    define_ref_immut_mut_access_concrete_methods!(membrane_potential, IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>, membrane_potential);
}
//endregion


//region Collection
pub struct PseudoVoxelModelNeuronCollection<CANQ: CorticalAreaNeuronQuantization> {
    membrane_potentials: [IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>],
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronModelContainerBaseTrait<CANQ> for PseudoVoxelModelNeuronCollection<CANQ> {
    type IndividualNeuronModelType = PseudoVoxelModelNeuron<CANQ>;

    fn get_neuron_value_max_index(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        todo!()
    }

    fn get_number_contained_neuron_values(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        todo!()
    }
}


impl<CANQ: CorticalAreaNeuronQuantization> NeuronModelDenseCollectionBaseTrait<CANQ> for PseudoVoxelModelNeuronCollection<CANQ> {
    define_ref_immut_mut_access_concrete_methods!(membrane_potentials, [IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>], membrane_potentials);
}

//endregion


