use ahash::{HashMap, HashMapExt};
use feagi_data::collections::spatial::contiguous_data::SpatialContiguousVector3D;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantizationFloat32;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiGlobalQuantizationStandard, FeagiIndexQuantization};
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_npu_dynamic_allocator::connectome_allocation_verifier::ConnectomeAllocationVerifier;
use feagi_npu_dynamic_allocator::genome_engine_map::GenomeEngineMap;
use feagi_npu_dynamic_allocator::npu_request::npu_request::NPURequest;

type StandardNeuronQuantization = <FeagiGlobalQuantizationStandard as FeagiIndexQuantization>::NeuronIndexCountQuant;

pub struct DynamicNPU {



    connectome_allocation_verifier: ConnectomeCacheWrapped,
}

impl DynamicNPU {

}



enum ConnectomeCacheWrapped {
    Standard(GenomeEngineMap<FeagiGlobalQuantizationStandard>),
}


