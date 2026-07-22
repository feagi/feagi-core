//! Neuron Models need to implement support of at least one (but can be multiple) cortical layout
//! traits. They define how a neuron model interacts within the context of a specified neuron
//! layout

use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_index::BurstIndex;
use crate::neuron::genome_interface::cortical_area_spawner::{DimensionalCorticalAreaSpawner, UniformCorticalAreaSpawner};
use crate::neuron::neuron_model::NeuronModel;
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;

//region Dimensional

/// Extend `NeuronModel` to denote that the model can function on dimensional cortical areas
pub trait NeuronModelDimensionalLayoutSupport<FIQ, NMQ>: NeuronModel<FIQ, NMQ>
where // NOTE: These all should be extended for the given neuron model!
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
{
    
    /// Dimensional Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false.
    fn process_incoming_potential_for_dimensional_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool;
    
    /// Return a default `CorticalAreaData` for this neuron model to use when creating a new cortical area
    fn default_dimensional_area_cortical_data() -> Self::CorticalData;
    
    /// Default `DimensionalCorticalAreaSpawner` for creating this model with its initial parameters
    fn default_dimensional_area_spawner() -> impl DimensionalCorticalAreaSpawner {
        UniformCorticalAreaSpawner::new(
            Self::CorticalData::default(),
            Self::NeuronData::default(),
        )
    }
}

//endregion

// TODO formless (similar as above but 