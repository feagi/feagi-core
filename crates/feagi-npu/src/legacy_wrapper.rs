use ahash::HashMap;
use feagi_data::collections::linear::contiguous::BitPackedVector;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_npu_common::wrapped_values::BurstIndex;
use feagi_npu_engines::rayon::engine::BurstEngineCpuRayon;
use feagi_npu_models::neuron_models::feagi_standard::quantization::FeagiStandardModelStandard32BitQuant;

pub struct NPULegacyBurstEngineWrapper {
    engine: BurstEngineCpuRayon<FeagiStandardModelStandard32BitQuant>
}

impl NPULegacyBurstEngineWrapper {

    pub fn new() -> Self {
        Self {
            engine: BurstEngineCpuRayon::new() // TODO init function must be externally usable
        }
    }

    pub fn run_burst(&mut self) -> (BurstIndex<u32>, HashMap<CorticalID, BitPackedVector<u32>>) {

    }
}