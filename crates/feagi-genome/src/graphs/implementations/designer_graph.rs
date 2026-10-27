use std::collections::{HashMap, HashSet};
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_area_container::{CorticalAreaContainer, CorticalAreaCachingEnabled};
use crate::graphs::feagi_graph::{CorticalCachingOf, CorticalIdentifierOf, FeagiGraph};
use crate::identifiers::brain_regions::BrainRegionID;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::graphs::components::element_identifiers::{ConcreteGraphElementIdentifiers, GraphElementIdentifiers};
use crate::graphs::components::graph_caching::GraphCachingEnabled;
use crate::graphs::components::npu_context::GraphNPUContextNone;

/// A dynamic Genome Graph that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGraph<FIQ: FeagiIndexQuantization> {
    genome_name: String,
    root_brain_region: (BrainRegionID, ()),
    brain_regions: HashMap<BrainRegionID, ()>,
    cortical_areas: HashMap<CorticalID, CorticalAreaContainer<FIQ, CorticalAreaCachingEnabled>>,

    // TODO NPU
    
    
    // TODO
    // connectivity rules (id -> rule)
    // cortical mappings (id -> cortical mapping set (mapping id -> mapping) )
    
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> DesignerGraph<FIQ> {
    pub fn new_now(genome_name: String) -> Self {
        Self { 
            genome_name,
            root_brain_region: (BrainRegionID::new_now(), ()),
            brain_regions:  Default::default(),
            cortical_areas: Default::default(),
            _p: core::marker::PhantomData, }
    }
}

impl<FIQ: FeagiIndexQuantization> FeagiGraph<FIQ> for DesignerGraph<FIQ> {
    type ElementIdentifiers = ConcreteGraphElementIdentifiers;
    type CachingStrategy = GraphCachingEnabled;
    type NPUContext = GraphNPUContextNone;

    fn get_cortical_area_map(&self) -> &HashMap<CorticalIdentifierOf<Self, FIQ>, CorticalAreaContainer<FIQ, CorticalCachingOf<Self, FIQ>>> {
        &self.cortical_areas
    }

    fn get_brain_region(&self, brain_region_id: ()) -> Option<&()> {
        todo!()
    }

    fn get_connectivity_rule(&self, connectivity_rule_id: ()) -> Option<&()> {
        todo!()
    }

    fn get_mapping_entry(&self, mapping_entry_id: ()) -> Option<&()> {
        todo!()
    }


}

impl<FIQ: FeagiIndexQuantization> DesignerGraph<FIQ> {
    
    
}




