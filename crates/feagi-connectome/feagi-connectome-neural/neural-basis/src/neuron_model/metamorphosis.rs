//! System for describe cortical area edits (not resizing). None of these should cause reallocations

use crate::neuron_model::neuron_pattern::pattern_certain_indexes::NeuronPatternCertainIndexes;
use crate::neuron_model::NeuronModel;
use crate::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use crate::quantization_levels::FeagiIndexQuantization;

// TODO wrapper enum to diff between NeuronModels

/// A selection of commands to edit a cortical area in place (no alloc / dealloc).
pub enum CorticalMetamorphosisType<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Edits genome exported cortical properties
    EditCorticalProperties {
        cortical_index: CorticalAreaEngineIndex<FIQ::NeuronIndexQuant>,
        cortical_properties: Option<NM::CorticalDataProperties>,
        cortical_neuron_variance: Option<NM::CorticalDataVariance>,
        is_psp_uniform: Option<bool>,
        psp_is_neuron_potential: Option<bool>
    },
    /// Sets some neurons as firing for the upcoming burst
    ForceFire {
        cortical_index: CorticalAreaEngineIndex<FIQ::NeuronIndexQuant>,
        force_firing: NeuronPatternCertainIndexes<FIQ, NM>
    },
    /// Forces neurons to be on or off and remain so until changed
    SetNeuronProbes {
        cortical_index: CorticalAreaEngineIndex<FIQ::NeuronIndexQuant>,
        probe_state: NeuronProbeState
    }
}

impl<FIQ, NM> CorticalMetamorphosisType<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Get what cortical index is being changed
    pub fn affecting_cortical_index(&self) -> &CorticalAreaEngineIndex<FIQ::NeuronIndexQuant> {
        match &self {
            CorticalMetamorphosisType::EditCorticalProperties { cortical_index, .. } => {
                cortical_index
            }
            CorticalMetamorphosisType::ForceFire { cortical_index, .. } => {
                cortical_index
            }
            CorticalMetamorphosisType::SetNeuronProbes { cortical_index, .. } => {
                cortical_index
            }
        }
    }
}

#[repr(u8)]
pub enum NeuronProbeState {
    NoProbe,
    ForceOn,
    ForceOff
}