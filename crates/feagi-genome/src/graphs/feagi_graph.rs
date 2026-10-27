use std::collections::HashMap;
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_area_container::{CorticalAreaContainer};
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::graphs::components::element_identifiers::{GraphElementIdentifiers};
use crate::graphs::components::graph_caching::GraphCaching;
use crate::graphs::components::npu_context::GraphNPUContext;



/// Represents the layout and (a variable amount of) data of a FEAGI brain
pub trait FeagiGraph<FIQ: FeagiIndexQuantization>
{
    /// What identifiers are used
    type ElementIdentifiers: GraphElementIdentifiers;
    
    /// What / if we cache some precompute values for rapid lookup, or not to save memory
    type CachingStrategy: GraphCaching;

    /// The types of data contained that is relevant to NPU communication / translation
    type NPUContext: GraphNPUContext;


    //region Reading Properties

    fn get_cortical_area_map(&self) -> &HashMap<CorticalIdentifierOf<Self, FIQ>, CorticalAreaContainer<FIQ, CorticalCachingOf<Self, FIQ>>>;
    
    fn get_brain_region(&self, brain_region_id: ()) -> Option<&()>;

    fn get_connectivity_rule(&self, connectivity_rule_id: ()) -> Option<&()>;

    fn get_mapping_entry(&self, mapping_entry_id: ()) -> Option<&()>;
    
    //region Default Impls
    
    fn get_cortical_area(&self, cortical_id: &CorticalIdentifierOf<Self, FIQ>) -> Option<&CorticalAreaContainer<FIQ, CorticalCachingOf<Self, FIQ>>> {
        
        self.get_cortical_area_map().get(cortical_id)
    }
    
    fn get_all_cortical_areas(&self) -> Vec<CorticalIdentifierOf<Self, FIQ>> {
        let all_cortical_ids = self.get_cortical_area_map()
            .iter()
            .map(|(id, _) |id.clone())
            .collect();
        all_cortical_ids
    }

    
    
    // TODO Multis

    //endregion


    //endregion
}

#[doc(hidden)]
/// The cortical area identifier type used by graph `G`
pub type CorticalIdentifierOf<G, FIQ> = <<G as FeagiGraph<FIQ>>::ElementIdentifiers as GraphElementIdentifiers>::CorticalIdentifier;

#[doc(hidden)]
/// The cortical area caching strategy used by graph `G`
pub type CorticalCachingOf<G, FIQ> = <<G as FeagiGraph<FIQ>>::CachingStrategy as GraphCaching>::CorticalCaching;






// Storage (stored per data or writer), Storage Light (writer only), runtime (from npu), amalgamation (stored per or writer, faux identifiers)

