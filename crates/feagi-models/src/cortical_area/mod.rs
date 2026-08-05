use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::QuantizedDecimalTrait;

pub mod neuron;
pub mod cortical_activity;
pub mod fire_ledger;
pub mod cortical_area_layout;
pub mod cortical_writer;
pub mod cortical_editor;
pub mod cortical_writer_by_model_quant;
pub mod implementations;

/// Defines a cortical area
#[derive(Clone, Copy, Debug)]
pub struct CorticalArea<MPQ: MembranePotentialQuantization> {
    /// Defines how potential is defined for a firing neurons
    pub post_cortical_potential: PostCorticalPotential<MPQ::MembranePotentialQuant>,
    /// Defines if the neuron output will be uniform across all outgoing synapses
    pub is_psp_uniform: bool,
    /// Probe setting, causes cortical area neurons to ignore any input or even run, freezing it in
    /// time. However, if in this moment a neuron was firing, it will be stuck in a firing state
    pub probe_cortical_area_input_disabled: bool,
    /// Probe setting, causes cortical area neurons to never express a firing state, regardless of
    /// what their actual firing state is
    pub probe_cortical_area_output_disabled: bool,

    // TODO neuron model stuff
}



/// Defines how potential is defined for a firing neuron in a cortical area
#[derive(Clone, Copy, Debug)]
pub enum PostCorticalPotential<MembranePotentialQuant: QuantizedDecimalTrait>
{
    MembraneDriven,
    Uniform(NeuronMembranePotential<MembranePotentialQuant>)
}