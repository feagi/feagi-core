use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::NeuronVoxelDensityIndex;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::neuron_models::neuron_model_quantization_encoding::NeuronModelTypeAndQuantization;
use crate::request_parameters::writers::cortical_neuron_writers::DimensionalCorticalNeuronWriter;

/// Add / Remove / Edit cortical areas
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersCorticalArea {
    AddCorticalArea(NPURequestParametersCorticalAreaCreate),
    // TODO Edit / Remove
}

// TODO Target Engine Index



/// Request Making a Cortical Area
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersCorticalAreaCreate 
{
    Interconnect {
        dimensions: NeuronVoxelDimensions<u64>,
        voxel_density: NeuronVoxelDensityIndex<u64>,
        cortical_id: CorticalID,
        neuron_model_type_and_quantization: NeuronModelTypeAndQuantization,
        specific_engine_index: Option<()>
    },
    // TODO other cortical types, including IO (we can have them without having agents)
} 

// TODO Editing cortical area (maybe split this to several types?)

// TODO Removing cortical areas

// TODO editing neurons  (take an iterator)

