use crate::synapse::synapse_flags::SynapseFlag;

pub struct NonplasticSynapseProperties<WeightQuant, PotentialQuant> {
    flags: SynapseFlag,
    weights: Weight,
    postsynaptic_potentials: Potential

}