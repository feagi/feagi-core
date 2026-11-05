use crate::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use crate::quantization_levels::FeagiIndexQuantization;
use crate::synapse_model::{SynapseBurstEngineDynamics, SynapseModel};


pub trait SynapseModelVoxelToVoxel: SynapseModel<
    SourceAreaLayoutTagRestriction=NeuronLayoutTagVoxel,
    DestinationAreaLayoutTagRestriction=NeuronLayoutTagVoxel
>
{

}

/// This marker trait is used to implement per synapse model dynamics per burst engine
/// backend. It is to be extended with a synapse subtype trait due to the radical difference
/// in synapse models
pub trait SynapseVoxelToVoxelBurstEngineDynamics<FIQ, SM>: SynapseBurstEngineDynamics<FIQ, SM>
where
    FIQ: FeagiIndexQuantization,
    SM: SynapseModel
{}