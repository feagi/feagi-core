use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::connectivity_rule::AxonBundleSynapseLocalIndex;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


pub trait ConnectivityRuleSynapseMapping<FIQ: FeagiIndexQuantization> {
    fn source_destination_synapse_mappings(
        synapse_index: &AxonBundleSynapseLocalIndex<FIQ::SynapseIndexCountQuant>
    ) -> SourceDestinationSynapseMapping<FIQ>;

    fn iter() -> impl ExactSizeIterator + 'static;
}

pub type SourceDestinationSynapseMapping<FIQ: FeagiIndexQuantization> = (
    CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>
);