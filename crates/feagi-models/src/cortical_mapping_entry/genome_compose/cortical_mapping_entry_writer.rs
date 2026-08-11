use crate::cortical_mapping_entry::synapse::cortical_mapping_entry_data::SynapseModelCorticalMappingEntryData;
use crate::cortical_mapping_entry::synapse::cortical_mapping_entry_properties::CorticalMappingEntryProperties;
use crate::cortical_mapping_entry::synapse::synapse_data::SynapseModelSynapseData;
use crate::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;
use crate::cortical_mapping_entry::synapse::synapse_properties::SynapseProperties;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;

pub trait SynapseModelCorticalWriter<SMQ, SMCMD, SMSD>
where
    SMQ: SynapseModelQuantization,
    SMCMD: SynapseModelCorticalMappingEntryData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
{
    /// Number of synapses needed
    fn number_synapses_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::SynapseIndexCountQuant, ()>; // TODO error!

    /// Handles writing the per synapse data and creating the properties.
    /// ALL MEMBERS are to be overwritten!
    fn write_to_synapse_region<FIQ: FeagiIndexQuantization>(
        self,
        cortical_data: &mut SMCMD,
        neuron_data: &mut [SMSD],
        neuron_properties: &mut [SynapseProperties], // TODO this is messy, we should find a way to get the 'impl iterator' thing to work
    ) -> Result<(CorticalMappingEntryProperties), ()>;
}

/// Root enum used to defining how a cortical mapping can be created. Enforces some universal methods.
/// By constraining model specific implementations to a generic sub enum, we can statically
/// create this easily!
pub enum RootSynapseModelCorticalWriter<SMQ, SMCMD, SMSD, SE>
where
    SMQ: SynapseModelQuantization,
    SMCMD: SynapseModelCorticalMappingEntryData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    SE: SynapseModelCorticalWriter<SMQ, SMCMD, SMSD>,
{
    /// In the case that we have a full set of data (IE from connectome loading), load the full
    /// data directly! Useful for overwriting / creating a new mapping
    CompleteRawData {
        _p: core::marker::PhantomData<SMQ>,
        cortical_mapping_entry_data: SMCMD,
        cortical_mapping_properties: CorticalMappingEntryProperties,
        synapse_data: Vec<SMSD>, // len should match what layout defines and properties
        synapse_properties: Vec<SynapseProperties>,
    },
    ModelSpecific(SE),
}

impl<SMQ, SMCMD, SMSD, SE> SynapseModelCorticalWriter<SMQ, SMCMD, SMSD> for RootSynapseModelCorticalWriter<SMQ, SMCMD, SMSD, SE>
where
    SMQ: SynapseModelQuantization,
    SMCMD: SynapseModelCorticalMappingEntryData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    SE: SynapseModelCorticalWriter<SMQ, SMCMD, SMSD>,
{
    fn number_synapses_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::SynapseIndexCountQuant, ()> {
        match self {
            RootSynapseModelCorticalWriter::CompleteRawData { synapse_data, .. } => {
                Ok(FIQ::SynapseIndexCountQuant::quant_from_usize_unchecked(synapse_data.len()))
            }
            RootSynapseModelCorticalWriter::ModelSpecific(SE) => SE.number_synapses_needed::<FIQ>(),
        }
    }

    fn write_to_synapse_region<FIQ: FeagiIndexQuantization>(
        self,
        current_cortical_mapping_entry_data: &mut SMCMD,
        current_synapse_data: &mut [SMSD],
        synapse_properties_out: &mut [SynapseProperties],
    ) -> Result<(CorticalMappingEntryProperties), ()> {
        match self {
            RootSynapseModelCorticalWriter::CompleteRawData {
                _p,
                cortical_mapping_entry_data,
                cortical_mapping_properties,
                synapse_data,
                synapse_properties,
            } => {
                *current_cortical_mapping_entry_data = cortical_mapping_entry_data;
                current_synapse_data.copy_from_slice(synapse_data.as_slice());
                for (dst, src) in synapse_properties_out.iter_mut().zip(synapse_properties.into_iter()) {
                    *dst = src;
                }
                Ok(cortical_mapping_properties)
            }
            RootSynapseModelCorticalWriter::ModelSpecific(model) => {
                model.write_to_synapse_region::<FIQ>(current_cortical_mapping_entry_data, current_synapse_data, synapse_properties_out)
            }
        }
    }
}
