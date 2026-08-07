use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer::SynapseModelCorticalWriter;
use crate::cortical_mapping_entry::synapse::cortical_mapping_entry_properties::CorticalMappingEntryProperties;
use crate::cortical_mapping_entry::synapse::synapse_data::EmptyPerSynapseData;
use crate::cortical_mapping_entry::synapse::synapse_properties::SynapseProperties;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::data::{UniformSynapseModelCorticalMappingEntryData, UniformSynapseMultiplier};
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelQuantization;

#[derive(Debug, Clone, Copy)]
pub enum UniformSynapseWriter<SMQ>
where
    SMQ: UniformSynapseModelQuantization
{
    Default  {uniform_weight: UniformSynapseMultiplier<SMQ::JunctionPotentialQuant>, _p: PhantomData<SMQ> }
}

/*
impl<SMQ> SynapseModelCorticalWriter<SMQ, UniformSynapseModelCorticalMappingEntryData<SMQ>, EmptyPerSynapseData> for UniformSynapseWriter<SMQ>
where
    SMQ: UniformSynapseModelQuantization
{
    fn number_synapses_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::SynapseIndexCountQuant, ()> {
        match
    }

    fn write_to_synapse_region<FIQ: FeagiIndexQuantization>(self, cortical_data: &mut SMCMD, neuron_data: &mut [SMSD], neuron_properties: &mut [SynapseProperties]) -> Result<(CorticalMappingEntryProperties), ()> {
        todo!()
    }
}

 */
