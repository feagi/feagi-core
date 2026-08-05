// TODO a lot of this is honestly proc macro work
//region Quantization

use crate::cortical_mapping_entry::synapse_model::synapse_model::SynapseModel;
use crate::cortical_mapping_entry::synapse_model::synapse_model_data::{EmptyPerSynapseData, SynapseCorticalMappingEntryData};
use crate::cortical_mapping_entry::synapse_model::synapse_model_quantization::{SynapseModelQuantization, SynapseModelQuantizationLevel};
use feagi_data::create_wrapped_quantized_decimal;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait};
use crate::cortical_mapping_entry::synapse_model::model_capabilities::source_fire_history::NeuronFireHistoryNone;
use crate::cortical_mapping_entry::synapse_model::model_generated::model_type_and_quantization::{SynapseModelTypeAndQuantizationNested, SynapseModelType};

pub trait PlasticSynapseModelQuantization: SynapseModelQuantization {
    const MODEL_QUANTIZATION_LEVEL: PlasticSynapseModelQuantizationLevel;
    // Multiplier quant uses the same quant as the In/ Out for synapse
}

//region Discrete Levels

#[derive(Default, Clone, Copy)]
pub struct PlasticSynapseModelStandardQuant;

impl PlasticSynapseModelQuantization for PlasticSynapseModelStandardQuant {
    const MODEL_QUANTIZATION_LEVEL: PlasticSynapseModelQuantizationLevel = PlasticSynapseModelQuantizationLevel::Standard;
}

impl SynapseModelQuantization for PlasticSynapseModelStandardQuant {
    type JunctionPotentialQuant = f32;
    const SYNAPSE_MODEL: SynapseModelType = SynapseModelType::Plastic;
    type QuantLevelType = PlasticSynapseModelQuantizationLevel;
    const SYNAPSE_QUANTIZATION: Self::QuantLevelType = PlasticSynapseModelQuantizationLevel::Standard;
    const NESTED_SYNAPSE_MODEL_AND_QUANTIZATION: SynapseModelTypeAndQuantizationNested =
        SynapseModelTypeAndQuantizationNested::Plastic(Self::SYNAPSE_QUANTIZATION);
    const USED_QUANTIZATION_LEVELS: &'static [DecimalQuantizationLevel] = &[DecimalQuantizationLevel::F32];
}

//endregion

// TODO Macro should eb generating this stuff

/// The quantization used by the Plastic Synapse Model
#[repr(u8)]
#[derive(Debug, Copy, Default, Clone, Hash, PartialEq, Eq)]
pub enum PlasticSynapseModelQuantizationLevel {
    #[default]
    Standard = 0,
}

impl SynapseModelQuantizationLevel for PlasticSynapseModelQuantizationLevel {
    // TODO copy some properties here
}

//endregion

//region Data

create_wrapped_quantized_decimal!(
    /// A multiplier synapse value, applies some scale to incoming signal
   pub PlasticSynapseMultiplier);

#[derive(Debug, Clone, Default)]
pub struct UniformSynapseModelCorticalMappingEntryData<SMQ>
where
    SMQ: PlasticSynapseModelQuantization,
{
    pub post_synaptic_multiplier: PlasticSynapseMultiplier<SMQ::JunctionPotentialQuant>,
}

impl<SMQ> SynapseCorticalMappingEntryData<SMQ> for UniformSynapseModelCorticalMappingEntryData<SMQ> where SMQ: PlasticSynapseModelQuantization {}

impl<SMQ> UniformSynapseModelCorticalMappingEntryData<SMQ>
where
    SMQ: PlasticSynapseModelQuantization,
{
    pub fn new(post_synaptic_multiplier: PlasticSynapseMultiplier<SMQ::JunctionPotentialQuant>) -> Self {
        Self { post_synaptic_multiplier }
    }
}

// No per synapse data

pub struct UniformSynapseModel<FIQ, SMQ>
where
    FIQ: FeagiIndexQuantization,
    SMQ: PlasticSynapseModelQuantization,
{
    _p: core::marker::PhantomData<(FIQ, SMQ)>,
}

impl<FIQ, SMQ> SynapseModel<FIQ, SMQ> for UniformSynapseModel<FIQ, SMQ>
where
    FIQ: FeagiIndexQuantization,
    SMQ: PlasticSynapseModelQuantization,
{
    type CorticalMappingEntryData = UniformSynapseModelCorticalMappingEntryData<SMQ>;
    type SynapseData = EmptyPerSynapseData;
    type SourceFireHistory = NeuronFireHistoryNone;

    fn synapse_process_incoming_signal(
        incoming_potential: &NeuronMembranePotential<SMQ::JunctionPotentialQuant>,
        mapping_entry_data: &Self::CorticalMappingEntryData,
        _source_fire_history: &Self::SourceFireHistory,
    ) -> NeuronMembranePotential<SMQ::JunctionPotentialQuant> {
        let incoming = incoming_potential.deref();
        let multiplier = mapping_entry_data.post_synaptic_multiplier.deref();
        
        NeuronMembranePotential::new(incoming *  multiplier)
    }
}

//endregion
