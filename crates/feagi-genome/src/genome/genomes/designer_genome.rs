use crate::cortical_area::CorticalAreaTypeMap;
use crate::basis::specifications::brain_regions::BrainRegionID;
use std::collections::{HashMap, HashSet};
use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::genome::containers::brain_region::{BrainRegionContainerRootRuntime, BrainRegionContainerRuntime};
use crate::genome::containers::cortical_area::CorticalAreaContainersDesigner;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

/// The name given to the root brain region of a new genome
const ROOT_BRAIN_REGION_NAME: &str = "Root";

/// A dynamic Genome that stores genomic level data for the running feagi brain and acts
/// as the interface and translator for modifications to that brain directly on the genome and
/// through the NPU
pub struct DesignerGenome<FIQ: FeagiIndexQuantization> {
    genome_name: String,
    root_brain_region_id: BrainRegionID,
    brain_regions: HashMap<BrainRegionID, BrainRegionContainerRuntime>,
    cortical_areas: CorticalAreaTypeMap<CorticalAreaContainersDesigner<FIQ>>,

    // TODO
    // connectivity rules (id -> rule)
    // cortical mappings (id -> cortical mapping set (mapping id -> mapping) )
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> DesignerGenome<FIQ> {

    /// A new, empty genome, holding only its root brain region
    pub fn init(genome_name: String) -> Self {
        let mut genome = Self {
            genome_name,
            root_brain_region_id: BrainRegionID::new_now(),
            brain_regions: Default::default(),
            cortical_areas: Default::default(),
            _p: Default::default(),
        };
        genome.ensure_root_brain_region();
        genome
    }

    /// Adds the root brain region, if there isn't one already
    fn ensure_root_brain_region(&mut self) {
        let root_id = self.root_brain_region_id;
        self.brain_regions.entry(root_id).or_insert_with(|| {
            BrainRegionContainerRuntime::Root(BrainRegionContainerRootRuntime {
                metadata: GenomeObjectMetadata {
                    name: ROOT_BRAIN_REGION_NAME.to_string(),
                    description: String::new(),
                },
                positioning: GenomeObjectPositioning {
                    position_2d: SignedIntegerSpatial::new_from_array([0, 0]),
                    position_3d: SignedIntegerSpatial::new_from_array([0, 0, 0]),
                },
                ancestor_id: root_id.into(),
                contained_areas: HashSet::new(),
                contained_regions: HashSet::new(),
            })
        });
    }

    //region Reading

    pub fn genome_name(&self) -> &str {
        &self.genome_name
    }

    /// The brain region every other is within
    pub fn root_brain_region_id(&self) -> BrainRegionID {
        self.root_brain_region_id
    }

    pub fn brain_region(&self, brain_region_id: &BrainRegionID) -> Option<&BrainRegionContainerRuntime> {
        self.brain_regions.get(brain_region_id)
    }

    pub fn brain_regions(&self) -> &HashMap<BrainRegionID, BrainRegionContainerRuntime> {
        &self.brain_regions
    }

    pub fn cortical_areas(&self) -> &CorticalAreaTypeMap<CorticalAreaContainersDesigner<FIQ>> {
        &self.cortical_areas
    }

    //endregion

    //region Editing, only through genome edit steps

    pub(crate) fn brain_regions_mut(&mut self) -> &mut HashMap<BrainRegionID, BrainRegionContainerRuntime> {
        &mut self.brain_regions
    }

    pub(crate) fn cortical_areas_mut(&mut self) -> &mut CorticalAreaTypeMap<CorticalAreaContainersDesigner<FIQ>> {
        &mut self.cortical_areas
    }

    //endregion
}
