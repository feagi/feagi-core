use crate::burst_engines::base::BaseBurstEngine;
use crate::fire_candidate_list::FireCandidateListRam;
use crate::fire_queue::FireQueueRam;
use crate::neuron::dimensional_neurons::core_neurons::CoreNeuronAllocRAMStorage;
use crate::neuron::dimensional_neurons::inter_neurons::InterNeuronAllocRAMStorage;
use crate::neuron::dimensional_neurons::motor_neurons::MotorNeuronAllocRAMStorage;
use crate::neuron::dimensional_neurons::sensory_neurons::SensoryNeuronAllocRAMStorage;
use crate::quantizables::NPUDataQuantization;
use crate::synapse::non_plastic_dimensional::NonplasticDimensionalSynapseAllocRAMStorage;

pub trait RAMBurstEngine<Q: NPUDataQuantization>: BaseBurstEngine<Q>
{
    fn execute_burst(&self,
                     fire_queue: &mut FireQueueRam<Q::NeuronIndexQuant>,
                     fire_candidate_list: &mut FireCandidateListRam<Q::NeuronIndexQuant>,
                     core_neurons: &mut CoreNeuronAllocRAMStorage<Q>,
                     sensory_neurons: &mut SensoryNeuronAllocRAMStorage<Q>,
                     motor_neurons: &mut MotorNeuronAllocRAMStorage<Q>,
                     inter_neurons: &mut InterNeuronAllocRAMStorage<Q>,
                     synapse_dimensional_nonplastic: &mut NonplasticDimensionalSynapseAllocRAMStorage<Q>
    );
}