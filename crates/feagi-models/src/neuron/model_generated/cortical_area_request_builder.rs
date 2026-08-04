use crate::connectome_requests::connectome_request::{ConnectomeRequest, ConnectomeRequestType};
use crate::neuron::models::feagi_advanced::FeagiAdvancedModelQuantizationLevel;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronVoxelDensityIndex};
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantizationGenomic, FeagiIndexQuantization, FeagiIndexQuantizationLevel};
use feagi_genomic_context::cortical_area::CorticalID;
use crate::neuron::model_generated::model_type_and_quantization::NeuronModelTypeAndQuantizationNested;
// TODO these should be generated

// TODO we shouldnt be defining cortical ID here when creating an area

type DimensionsGenomeQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

pub enum CorticalAreaRequest
{


    DeleteCorticalArea{id: CorticalID},
}
