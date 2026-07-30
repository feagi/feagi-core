use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::engines::rayon::data::RayonEngineData;

pub struct RayonBurstEngine<FIQ: FeagiIndexQuantization> {
    data: RayonEngineData<FIQ>
}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    pub fn new() -> Self {
        todo!()
    }
}
