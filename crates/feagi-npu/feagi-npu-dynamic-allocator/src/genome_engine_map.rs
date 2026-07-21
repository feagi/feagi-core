use std::collections::HashMap;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_genomic::feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::wrapped_indexes::CorticalEngineIndex;
use crate::npu_state_manager::burst_engine_context::burst_engine_context::BurstEngineIndex;

/// Maps various Genome lookups to the NPU. Also used for some initial request validity checking
pub trait GenomeEngineMap<FIQ: FeagiIndexQuantization> {

    /*
    /// What to add to a cortical area engine index to get the cortical area connectome index
    fn get_connectome_cortical_area_index_offset(&self) -> &FIQ::CorticalAreaIndexCountQuant;

    /// what to add to a cortical area engine index to get the cortical area connectome index
    fn get_connectome_cortical_area_index_offset_mut(&self) -> &mut FIQ::CorticalAreaIndexCountQuant;
     */

    /// From the cortical ID, try to get the cortical area engine index and the engine it resides in
    fn try_get_engine_and_cortical_engine_index(&self, cortical_id: &CorticalID) -> Result<(CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, BurstEngineIndex), ()>; // TODO Error
    
    


}

/// A more optimized / simple wrapper for when there is only ever a single burst engine
pub struct GenomeEngineMapSingleEngine<FIQ: FeagiIndexQuantization>
{
    cortical_id_lookup: HashMap<CorticalID, CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    mapping_entry_id_lookup: HashMap<u64, CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>>, // TODO synapse
}

impl<FIQ: FeagiIndexQuantization> GenomeEngineMapSingleEngine<FIQ>
{
    pub fn new() -> Self {
        Self {
            cortical_id_lookup: HashMap::new(),
            mapping_entry_id_lookup: HashMap::new(),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> GenomeEngineMap<FIQ> for GenomeEngineMapSingleEngine<FIQ> {

    fn try_get_engine_and_cortical_engine_index(&self, cortical_id: &CorticalID) -> Result<(CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, BurstEngineIndex), ()> {
        let result = self.cortical_id_lookup.get(cortical_id).ok_or(Err(()))?;
        Ok((result.clone(), 0)) // engine index always 0
    }
}