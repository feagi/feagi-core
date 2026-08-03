use crate::connectome_requests::connectome_request::{ConnectomeRequest, ConnectomeRequestType};
use crate::neuron::models::feagi_advanced::FeagiAdvancedModelQuantizationLevel;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronVoxelDensityIndex};
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiGlobalQuantizationGenomic, FeagiIndexQuantization, FeagiIndexQuantizationLevel};
use feagi_genomic_context::cortical_area::CorticalID;
use crate::neuron::model_generated::model_type_and_quantization::NeuronModelTypeAndQuantizationNested;
// TODO these should be generated

// TODO we shouldnt be defining cortical ID here when creating an area

type DimensionsGenomeQuant = <FeagiGlobalQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

pub enum CorticalAreaRequest
{
    NewDimensionalCorticalArea
    {
        dimensions: NeuronVoxelDimensions<<FeagiGlobalQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>,
        neurons_per_voxel: NeuronVoxelDensityIndex<<FeagiGlobalQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>,
        model_and_quant: NeuronModelTypeAndQuantizationNested,
        id: CorticalID
    },
    NewFormlessCorticalArea
    {
        neuron_count: <FeagiGlobalQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant,
        id: CorticalID
    },
    EditCorticalAreaCorticalFlags
    {

    },

    DeleteCorticalArea{id: CorticalID},
}









pub struct CorticalAreaRequestBuilder {
    index_level: FeagiIndexQuantizationLevel,
}

impl CorticalAreaRequestBuilder {
    #[doc(hidden)]
    pub(crate) fn new(index_level: FeagiIndexQuantizationLevel) -> Self {
        CorticalAreaRequestBuilder { index_level }
    }

    /// Request is related to a cortical area of the "feagi advanced" neuron model
    pub fn feagi_advanced(self) -> FeagiAdvancedCreatorRequestBuilder {
        FeagiAdvancedCreatorRequestBuilder {
            index_level: self.index_level,
        }
    }
}

pub struct FeagiAdvancedCreatorRequestBuilder {
    index_level: FeagiIndexQuantizationLevel,
}

impl FeagiAdvancedCreatorRequestBuilder {
    // TODO validate `quantization` against `self.index_level` once cross-quantization checks exist
    pub fn create_dimensional(
        self,
        cortical_id: u64, // CorticalID can be represented as a u64
        quantization: FeagiAdvancedModelQuantizationLevel,
        dimensions: NeuronVoxelDimensions<u64>,
        neurons_per_voxel: NeuronVoxelDensityIndex<u64>,
    ) -> Result<ConnectomeRequest, ()> {
        let dimensions = DimensionalCorticalArea4DDimensions::new_unchecked(
            *dimensions.get_x(),
            *dimensions.get_y(),
            *dimensions.get_z(),
            neurons_per_voxel,
        );

        Ok(ConnectomeRequest {
            index_level: self.index_level,
            request_type: ConnectomeRequestType::CorticalAreaAddDimensional(cortical_id, dimensions),
        })
    }

    // TODO formless areas need their own neuron-count representation once the connectome
    // request model grows one; for now only the cortical ID is carried through.
    pub fn create_formless(
        self,
        cortical_id: u64, // CorticalID can be represented as a u64
        _quantization: FeagiAdvancedModelQuantizationLevel,
        _number_neurons: u64,
    ) -> Result<ConnectomeRequest, ()> {
        Ok(ConnectomeRequest {
            index_level: self.index_level,
            request_type: ConnectomeRequestType::CorticalAreaAddFormless(cortical_id),
        })
    }

    pub fn delete(self, cortical_id: u64) -> ConnectomeRequest {
        ConnectomeRequest {
            index_level: self.index_level,
            request_type: ConnectomeRequestType::CorticalAreaDelete(cortical_id),
        }
    }
}
