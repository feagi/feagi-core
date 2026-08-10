use crate::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer::SynapseModelCorticalWriter;
use crate::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::UniformWriter;
use crate::cortical_mapping_entry::synapse::cortical_mapping_entry_properties::CorticalMappingEntryProperties;
use crate::cortical_mapping_entry::synapse::synapse_data::EmptyPerSynapseData;
use crate::cortical_mapping_entry::synapse::synapse_properties::SynapseProperties;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::data::{
    UniformSynapseModelCorticalMappingEntryData, UniformSynapseMultiplier,
};
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::{
    UniformSynapseModelQuantization, UniformSynapseModelStandardQuant,
};
use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;

#[derive(Debug, Clone, Copy)]
pub enum UniformSynapseWriter<SMQ>
where
    SMQ: UniformSynapseModelQuantization,
{
    Default {
        /// How many synapses the resolved doublet iterator yields. Supplied by the engine, which is
        /// the only place that can apply both cortical area layouts to the genomic doublet.
        number_synapses: usize,
        uniform_weight: UniformSynapseMultiplier<SMQ::JunctionPotentialQuant>,
        propagation_delay: u16,
        is_inhibitory: bool,
        _p: PhantomData<SMQ>,
    },
}

impl UniformSynapseWriter<UniformSynapseModelStandardQuant> {
    /// Lifts a genome side mapping request into the writer the engine hands to
    /// [`SynapseModelCorticalWriter`]. `number_synapses` comes from the doublet iterator once both
    /// cortical area layouts have been applied, so it cannot be derived here.
    pub fn from_genomic_writer(writer: &UniformWriter, number_synapses: usize) -> Self {
        match writer {
            UniformWriter::Standard {
                uniform_weight,
                propagation_delay,
                is_inhibitory,
                ..
            } => UniformSynapseWriter::Default {
                number_synapses,
                uniform_weight: *uniform_weight,
                propagation_delay: *propagation_delay,
                is_inhibitory: *is_inhibitory,
                _p: PhantomData,
            },
        }
    }
}

impl<SMQ> SynapseModelCorticalWriter<SMQ, UniformSynapseModelCorticalMappingEntryData<SMQ>, EmptyPerSynapseData> for UniformSynapseWriter<SMQ>
where
    SMQ: UniformSynapseModelQuantization,
{
    fn number_synapses_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::SynapseIndexCountQuant, ()> {
        match self {
            UniformSynapseWriter::Default { number_synapses, .. } => {
                // Bounds checked rather than truncated: a count that does not fit the synapse
                // quantization would silently mis-size the engine's synapse region.
                FIQ::SynapseIndexCountQuant::quant_try_from_usize(*number_synapses).map_err(|_| ())
            }
        }
    }

    fn write_to_synapse_region<FIQ: FeagiIndexQuantization>(
        self,
        cortical_data: &mut UniformSynapseModelCorticalMappingEntryData<SMQ>,
        _synapse_data: &mut [EmptyPerSynapseData],
        synapse_properties: &mut [SynapseProperties],
    ) -> Result<(CorticalMappingEntryProperties), ()> {
        match self {
            UniformSynapseWriter::Default {
                number_synapses,
                uniform_weight,
                propagation_delay,
                is_inhibitory,
                ..
            } => {
                if synapse_properties.len() != number_synapses {
                    return Err(()); // region was sized against a different count than this writer reports
                }

                // The weight is shared by the whole mapping entry, so it lives on the entry data.
                // Per synapse data stays `EmptyPerSynapseData`: this model stores nothing per synapse.
                *cortical_data = UniformSynapseModelCorticalMappingEntryData::new(uniform_weight);

                for properties in synapse_properties.iter_mut() {
                    *properties = SynapseProperties::default();
                }

                Ok(CorticalMappingEntryProperties {
                    propagation_delay,
                    is_inhibitory,
                })
            }
        }
    }
}
