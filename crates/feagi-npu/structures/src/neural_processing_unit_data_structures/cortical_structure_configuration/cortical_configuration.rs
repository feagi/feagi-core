use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationLevel, FeagiGlobalQuantization, FeagiGlobalQuantizationLevel};
use crate::neural_processing_unit_data_structures::cpu_wrappers::indexes_global::NPUNeuronMembranePotential;
use crate::neural_processing_unit_data_structures::cpu_wrappers::cortical_spatial::{NPUCorticalAreaDimensions, NPUNeuronIndexCorticalLocal};


/// Base trait for Cortical configuration, which is simply general details about a cortical area
/// not related to the neuron model, but needed broadly for compute and context, This is NOT
/// extendable by any neuron model!
pub trait CorticalConfigurationBase<FGQ, CPQ>:
PDIElement
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    // contains the post_synaptic_potential_base, post_synaptic_potential_should_be_uniform,
    // is_postsynaptic_potential_drive_by_membrane_potential, number_active_neurons_this_burst

    // Only contain usable data if on the CPU

    // Number of neurons contained should be accessible
}

pub trait CorticalConfigurationDimensional<FGQ, CPQ>:
CorticalConfigurationBase<FGQ, CPQ>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    // Dimensions of cortical area (4D) should be accessible
}

// TODO other types of cortical areas?

//region CPU implementations

// TODO we need to have traits for the CPU dimensional / etc stuff to properly support them in
// parameters

//region Dimensional


// TODO BitPackedBool for the bools!

#[repr(C)]
pub struct CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, const PADDING_END: usize>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    pub dimensions: NPUCorticalAreaDimensions<FGQ::NeuronIndexCountQuant>,
    pub number_active_neurons_this_burst: NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
    pub post_synaptic_potential_base: NPUNeuronMembranePotential<CPQ::NeuronPotentialQuant>,
    pub is_postsynaptic_potential_drive_by_membrane_potential: bool,
    pub post_synaptic_potential_should_be_uniform: bool,
    _padding: [u8; PADDING_END]
}

impl<FGQ, CPQ, const PADDING_END: usize> CorticalConfigurationBase<FGQ, CPQ> for CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, PADDING_END> where FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization, {}

impl<FGQ, CPQ, const PADDING_END: usize> PDIElement for CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, PADDING_END> where FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization, {}

impl<FGQ, CPQ, const PADDING_END: usize> PDITagGenericDevice for CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, PADDING_END> where FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization, {}

impl<FGQ, CPQ, const PADDING_END: usize> PDITagCPU for CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, PADDING_END> where FGQ: FeagiGlobalQuantization, CPQ: CorticalPotentialQuantization, {}

impl <FGQ, CPQ, const PADDING_END: usize>  CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, PADDING_END>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    /// Use this to calculate the amount of padding needed depending on the quantization
    pub const fn calculate_padding() -> usize
    where
        FGQ: FeagiGlobalQuantization,
        CPQ: CorticalPotentialQuantization
    {
        let global = FGQ::QUANTIZATION_LEVEL;
        let cortical = CPQ::QUANTIZATION_LEVEL;
        match (global, cortical) {
            (FeagiGlobalQuantizationLevel::Standard, CorticalPotentialQuantizationLevel::Float32) => {6}
        }
    }

    pub fn new(
        dimensions: NPUCorticalAreaDimensions<SpatialIndexDimensions4D<FGQ::NeuronIndexCountQuant>>,
        number_active_neurons_this_burst: NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
        post_synaptic_potential_base: NPUNeuronMembranePotential<CPQ>,
        is_postsynaptic_potential_drive_by_membrane_potential: bool,
        post_synaptic_potential_should_be_uniform: bool)
        -> CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, PADDING_END>
    {
        CorticalConfigurationDimensionalCPUQuant
        {
            dimensions,
            number_active_neurons_this_burst,
            post_synaptic_potential_base,
            is_postsynaptic_potential_drive_by_membrane_potential,
            post_synaptic_potential_should_be_uniform,
            _padding: [0; PADDING_END],
        }
    }
}



//endregion

// TODO other types of cortical areas?

//endregion

