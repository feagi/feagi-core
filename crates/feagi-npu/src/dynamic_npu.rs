use crate::engines::blocking::rayon::RayonBurstEngine;
use ahash::{HashMap, HashMapExt};
use feagi_data::neuron_voxels::collections::ContiguousVoxelVector;
use feagi_data::feagi_quantization_levels::cortical_potential_quantization::CorticalPotentialQuantizationFloat32;
use feagi_data::feagi_quantization_levels::feagi_index_quantization::FeagiGlobalQuantizationStandard;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_npu_dynamic_allocator::npu_request::npu_request::NPURequest;

pub struct DynamicNPU {
    // TODO for now, directly implicate the right quantized rayon burst engine
    burst_engine: RayonBurstEngine<FeagiGlobalQuantizationStandard>,
}

impl DynamicNPU {
    pub fn new() -> DynamicNPU {
        Self {
            burst_engine: RayonBurstEngine::new(),
        }
    }

    pub fn make_npu_request(&mut self, request: &NPURequest) -> Result<(), ()> {
        // TODO connect to stuff
        Ok(())
    }

    /// TEMP IMPLEMENTATION: takes in sensor data as a hashmap cortical ID to voxel vectors, and returns a tuple of a hashmap of motor cortical ID to motor data, and a hashmap of all cortical IDs to visualization data
    pub fn run_burst(
        &mut self,
        sensor_data: HashMap<CorticalID, ContiguousVoxelVector<FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32>>,
    ) -> (
        HashMap<CorticalID, ContiguousVoxelVector<FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32>>,
        HashMap<CorticalID, ContiguousVoxelVector<FeagiGlobalQuantizationStandard, CorticalPotentialQuantizationFloat32>>
    ) {
        // TODO implement
        return (HashMap::new(), HashMap::new());
    }
}
