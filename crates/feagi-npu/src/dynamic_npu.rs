use feagi_data::quantization_levels::feagi_index_quantization::{FeagiGlobalQuantizationAbsurd, FeagiGlobalQuantizationStandard, FeagiIndexQuantization};
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutDimensional;
use feagi_npu_dynamic_allocator::genome_engine_map::{GenomeEngineMap, GenomeEngineMapSingleEngine};
use feagi_npu_dynamic_allocator::npu_request::request_type::NPURequest;
use feagi_npu_dynamic_allocator::npu_request::parameters::cortical_area::{NPURequestParametersCorticalArea, NPURequestParametersCorticalAreaCreate};
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_models::neuron::models::feagi_advanced::model::FeagiAdvancedModel;
use feagi_models::neuron::models_shared_traits::model::NeuronModel;

type StandardNeuronQuantization = <FeagiGlobalQuantizationAbsurd as FeagiIndexQuantization>::NeuronIndexCountQuant; // TODO swap from absurd

pub struct DynamicNPU {
    connectome_allocation_verifier: ConnectomeCacheWrapped,
    rayon_burst_engine: // TODO swap to the `VectorBurstEngineGroup`
}

impl DynamicNPU {
    pub fn new() -> Self {
        Self {
            connectome_allocation_verifier: ConnectomeCacheWrapped::StandardSingleEngine(GenomeEngineMapSingleEngine::new()),
        }
    }

    pub fn take_request(&mut self, request: NPURequest) -> Result<(), ()> {
        match request {
            NPURequest::BurstEngine(b) => {
                todo!()
            }
            NPURequest::CorticalArea(c) => {
                match c {
                    NPURequestParametersCorticalArea::AddCorticalArea(req) => {
                        // TODO check cortical area not already exist (we should move out of pure enum for this)
                        match req {
                            NPURequestParametersCorticalAreaCreate::Interconnect {
                                dimensions,
                                voxel_density,
                                cortical_id,
                                neuron_model_type_and_quantization,
                                specific_engine_index,
                            } => {
                                let cortical_dim: DimensionalCorticalArea4DDimensions<FeagiGlobalQuantizationAbsurd> = DimensionalCorticalArea4DDimensions::new_from_usizes_unchecked(
                                    dimensions.get_x().quant_to_usize(),
                                    dimensions.get_y().quant_to_usize(),
                                    dimensions.get_z().quant_to_usize(),
                                    voxel_density.quant_to_usize()
                                );

                                let layout: CorticalAreaLayoutDimensional<FeagiGlobalQuantizationAbsurd> = CorticalAreaLayoutDimensional {
                                    dimensions: cortical_dim.clone()
                                };

                                let writer = FeagiAdvancedModel::default_neuron_writer_dimensional_layout_cortical_area(cortical_dim).unwrap();



                            }
                        }
                    }
                    NPURequestParametersCorticalArea::EditCorticalAreaProperties() => {
                        todo!()
                    }
                    NPURequestParametersCorticalArea::DeleteCorticalArea() => {
                        todo!()
                    }
                }
            }
            NPURequest::Mapping(m) => {
                todo!()
            }
            NPURequest::GenomeDebug() => {
                todo!()
            }
        }
    }
}

enum ConnectomeCacheWrapped {
    StandardSingleEngine(GenomeEngineMapSingleEngine<FeagiGlobalQuantizationStandard>),
}
