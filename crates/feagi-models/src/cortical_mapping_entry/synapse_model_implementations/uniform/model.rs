use crate::neuron_model::cortical_area::neuron_history::implementations::none::NeuronModelNoNeuronHistory;
use crate::cortical_mapping_entry::synapse::synapse_data::EmptyPerSynapseData;
use crate::cortical_mapping_entry::synapse::synapse_model::SynapseModel;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::data::UniformSynapseModelCorticalMappingEntryData;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelQuantization;
use feagi_data::neurons::neuron_potentials::neuron::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct UniformSynapseModel<FIQ, SMQ>
where
    FIQ: FeagiIndexQuantization,
    SMQ: UniformSynapseModelQuantization,
{
    _p: core::marker::PhantomData<(FIQ, SMQ)>,
}

impl<FIQ, SMQ> SynapseModel<FIQ, SMQ> for UniformSynapseModel<FIQ, SMQ>
where
    FIQ: FeagiIndexQuantization,
    SMQ: UniformSynapseModelQuantization,
{
    type CorticalMappingEntryData = UniformSynapseModelCorticalMappingEntryData<SMQ>;
    type SynapseData = EmptyPerSynapseData;
    type SourceFireHistory = NeuronModelNoNeuronHistory;

    fn synapse_process_incoming_signal(
        incoming_potential: &NeuronMembranePotential<SMQ::JunctionPotentialQuant>,
        mapping_entry_data: &Self::CorticalMappingEntryData,
        _source_fire_history: &Self::SourceFireHistory,
    ) -> NeuronMembranePotential<SMQ::JunctionPotentialQuant> {
        let incoming = incoming_potential.deref();
        let multiplier = mapping_entry_data.post_synaptic_multiplier.deref();

        NeuronMembranePotential::new(incoming * multiplier)
    }
}
