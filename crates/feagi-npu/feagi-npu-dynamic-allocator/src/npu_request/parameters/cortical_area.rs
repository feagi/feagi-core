use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::NeuronVoxelDensityIndex;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::neuron::interfacing::model_and_quantization::NeuronModelTypeAndQuantization;
use crate::npu_request::npu_request::{IntoNPURequest, NPURequest};
use crate::request_parameters::writers::cortical_neuron_writers::DimensionalCorticalNeuronWriter;

/// Add / Remove / Edit cortical areas
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersCorticalArea {
    AddCorticalArea(NPURequestParametersCorticalAreaCreate),
    EditCorticalAreaProperties(), // TODO -> in here editing properties, resizing,
    DeleteCorticalArea(), // TODO
    // TODO Edit / Remove
}

// TODO Target Engine Index

impl IntoNPURequest for NPURequestParametersCorticalArea {
    fn into_npu_request(self) -> NPURequest {
        self.into()
    }
}

impl From<NPURequestParametersCorticalAreaCreate> for NPURequestParametersCorticalArea {
    fn from(v: NPURequestParametersCorticalAreaCreate) -> Self {
        Self::AddCorticalArea(v)
    }
}


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

impl IntoNPURequest for NPURequestParametersCorticalAreaCreate {
    fn into_npu_request(self) -> NPURequest {
        let a: NPURequestParametersCorticalArea = self.into();
        a.into()
    }
}



// TODO Editing cortical area (maybe split this to several types?)

// TODO Removing cortical areas

// TODO editing neurons  (take an iterator)

