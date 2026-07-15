use ahash::HashMap;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_npu_dynamic_allocator::npu_request::npu_request::NPURequest;

pub struct DynamicNPU{
    
}

impl DynamicNPU {
    
    pub fn new() -> DynamicNPU {
        
    }
    
    pub fn make_npu_request(&mut self, request: &NPURequest) -> Result<(), ()>
    {
        todo!()
    }
    
    pub fn run_burst(&mut self, sensor_data: &HashMap<CorticalID, V>)
}