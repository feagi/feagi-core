use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::NeuronVoxelDensityIndex;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::neuron_models::common_enums::NeuronModelTypeAndQuantization;

/// Add / Remove / Edit cortical areas
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersCorticalArea {
    AddCorticalArea,
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
        model_and_quantization: NeuronModelTypeAndQuantization,
        neuron_iterator: &,
        specific_engine_index: Option<()>
        // TODO Target Engine Index, General Parameters Quantization, Neuron Model, Neuron Model Parameters
    },
    // TODO other cortical types, including IO (we can have them without having agents)
} 

// TODO Editing cortical area (maybe split this to several types?)

// TODO Removing cortical areas

// TODO editing neurons  (take an iterator)

