use std::collections::HashMap;
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_area_container::{CorticalAreaContainer, CorticalAreaRuntimeCache};
use crate::graphs::feagi_graph::FeagiGraph;
use crate::identifiers::brain_regions::BrainRegionID;
use crate::basis::cortical_area::cortical_id::CorticalID;
use crate::graphs::components::element_identifiers::ConcreteGraphElementIdentifiers;
use crate::graphs::components::npu_context::GraphNPUContextNone;

/// A dynamic Genome Graph that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGraph<FIQ: FeagiIndexQuantization> {
    genome_name: String,
    root_brain_region: (BrainRegionID, ()),
    brain_regions: HashMap<BrainRegionID, ()>,
    cortical_areas: HashMap<CorticalID, CorticalAreaContainer<FIQ, CorticalAreaRuntimeCache>>,

    // TODO NPU
    
    
    // TODO
    // connectivity rules (id -> rule)
    // cortical mappings (id -> cortical mapping set (mapping id -> mapping) )
    
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> DesignerGraph<FIQ> {
    pub fn new(genome_name: String) -> Self {
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
    type NPUContext = GraphNPUContextNone;
}

impl<FIQ: FeagiIndexQuantization> DesignerGraph<FIQ> {
    
    
}




