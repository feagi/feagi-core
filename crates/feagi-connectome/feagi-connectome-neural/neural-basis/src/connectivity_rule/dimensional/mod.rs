use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::connectivity_rule::{AxonBundleSynapseLocalIndex, ConnectivityRule};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


pub trait ConnectivityRuleDimensional<FIQ: FeagiIndexQuantization>:
ConnectivityRule<FIQ>
{
    
}









pub trait ConnectivityRuleSynapseMapping<FIQ: FeagiIndexQuantization> {
    fn source_destination_synapse_mappings(
        synapse_index: &AxonBundleSynapseLocalIndex<FIQ::SynapseIndexCountQuant>
    ) -> SourceDestinationSynapseMapping<FIQ>;

    fn iter() -> impl Iterator + 'static;
}




pub type SourceDestinationSynapseMapping<FIQ: FeagiIndexQuantization> = (
    CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>
);