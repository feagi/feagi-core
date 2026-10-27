use feagi_basis::feagi_basis_proc_common::define_request_category_group;
use super::categories::prelude::*;



// The requester and responder of every request category the genome manager serves. The
// responders are held by the genome manager worker itself, the requesters can be cloned and
// handed to whatever needs to make requests of those categories
define_request_category_group! {
    group_name: GenomeManager,
    categories: {
        burst_engine: BurstEngineRequestCategory,
        connectome: ConnectomeRequestCategory,
        cortical_area: CorticalAreaRequestCategory,
        cortical_mapping: CorticalMappingRequestCategory,
        genome: GenomeRequestCategory,
        morphology: MorphologyRequestCategory,
        region: RegionRequestCategory,
    }
}
