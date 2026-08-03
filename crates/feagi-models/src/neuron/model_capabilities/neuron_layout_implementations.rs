//! Neuron Models need to implement support of at least one (but can be multiple) cortical layout
//! traits. They define how a neuron model interacts within the context of a specified neuron
//! layout

use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::connectome_requests::properties::{UniversalCorticalAreaProperties, UniversalNeuronProperties};
use crate::neuron::cortical_area_layout::CorticalAreaLayoutDimensional;
use crate::wrapped_indexes::BurstIndex;
use crate::neuron::neuron_model::NeuronModel;
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;

/// Extend `NeuronModel` to denote that the model can function on dimensional cortical areas
pub trait DimensionalNeuronModel<FIQ, NMQ>: NeuronModel<FIQ, NMQ>
where // NOTE: These all should be extended for the given neuron model!
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
{
    
    /// Dimensional Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false.
    fn process_incoming_potential_for_dimensional_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexQuant>,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool;
}

/// This is the trait that all cortical area creators have to be able to export. Not directly creatable
/// from the outside, but instead from the inside
trait DimensionalNewCorticalAreaWriter<FIQ, NMQ, DNM>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
    DNM: DimensionalNeuronModel<FIQ, NMQ>,
{
    /// Get what the properties of the cortical area itself should be for writing
    fn get_cortical_area_properties(&self) -> &UniversalCorticalAreaProperties;

    /// For each neuron, what should the
    fn get_per_neuron_properties(&self, dimensional_data: &CorticalAreaLayoutDimensional<FIQ>) -> impl Iterator<Item=UniversalNeuronProperties>;

    /// Set cortical values to write
    fn set_cortical_model_data_values(&self, dimensional_data: &CorticalAreaLayoutDimensional<FIQ>, write_target: &mut DNM::CorticalData);

    /// Write neuron data for cortical area
    fn set_per_neuron_model_data_values(&self, dimensional_data: &CorticalAreaLayoutDimensional<FIQ>, write_target: &mut [DNM::CorticalData]);

}




/// Extend `NeuronModel` to denote that the model can function on formless cortical areas
pub trait FormlessNeuronModel<FIQ, NMQ>: NeuronModel<FIQ, NMQ>
where // NOTE: These all should be extended for the given neuron model!
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
{

    /// Formless Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false.
    fn process_incoming_potential_for_formless_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        cortical_neuron_count: &FIQ::NeuronIndexQuant,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool;
}