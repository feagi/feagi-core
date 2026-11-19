use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use crate::genome::genomes::brain_region::{BrainRegionGenomeContext, BrainRegionContainer, BrainRegionContainerRef, BrainRegionHierarchyContextRoot, BrainRegionSet, BrainRegionSetError};
use crate::genome::genomes::cortical_area::{CorticalAreaSet, GenomeCorticalAreaContext};

pub mod designer;
pub mod brain_region;
pub mod cortical_area;


/// Allows access to internal data states of a Genome Implementation
pub trait GenomePublicInterface<FIQ: FeagiIndexQuantization> {
    
    type BrainRegionSet: BrainRegionSet<BrainRegionContext=Self::BrainRegionContext>;
    type BrainRegionContext: BrainRegionGenomeContext;
    type CorticalAreaSet: CorticalAreaSet<FIQ>;
    
    fn get_brain_region_set(&self) -> &Self::BrainRegionSet;
    
    fn get_cortical_area_set(&self) -> &Self::CorticalAreaSet;

    //fn get_connectivity_rule_context(&self) -> Result<(), ()>;

    //fn get_cortical_mapping_set(&self) -> Result<(), ()>;

    //fn get_mapping_entry(&self) -> Result<(), ()>;

    //region default impls

    fn contains_brain_region_id(&self, brain_id: &BrainRegionID) -> bool {
        todo!()
    }

    fn get_root_region_container(&self) -> &BrainRegionContainer<Self::BrainRegionContext, BrainRegionHierarchyContextRoot> {
        self.get_brain_region_set().get_root_brain_region_container()
    }

    fn get_brain_region_context(&self, region_id: &BrainRegionID) -> Result<BrainRegionContainerRef<Self::BrainRegionContext>, GenomeError> {
        let set = self.get_brain_region_set();
        set.get_brain_region_ref(region_id).ok_or_else(|| GenomeError::BrainRegion(BrainRegionSetError::BrainRegionIDNotFound))
    }
    
    fn contains_cortical_area_id(&self, cortical_identifier: CorticalIdentifier) -> bool { todo!() }

    //fn get_brain_region_ids_contained_by_region(&self) -> &HashSet<BrainRegionID>;

    //fn get_area_ids_contained_by_region(&self) -> &HashSet<CorticalIdentifier>;

    //fn get_cortical_area_context(&self, cortical_id: &CorticalIdentifier) -> Result<(), ()>;
    
    // TODO nested totals (cortical areas underneath brain region)?

    //endregion
}

/// Allows direct modification to genome structures that are accompanied by connectome level
/// changes. Since this often needs per implementation handling (including verification) this
/// must not be public. From outside the crate, these types of changes should only be done by
/// an external verifying system such as 'GenomeEditSequence'
pub(crate) trait GenomeInternalInterface<FIQ: FeagiIndexQuantization,>: GenomePublicInterface<FIQ> {
    
    // NOTE: In the case of designer, many of these changes should be made after any
    // applicable changes where made to the connectome with success first
    
    /// Gets brain regions in a mutable context. Cant be public since its possible to do things
    /// like drop cortical areas
    fn get_brain_region_contexts_mut(&mut self) -> &mut Self::BrainRegionSet;

    /// Gets cortical areas in a mutable context. Cant be public since its possible to do things
    /// like resize the area
    fn get_cortical_area_set_mut(&self) -> &mut Self::CorticalAreaSet;

    //fn get_connectivity_rule_contexts_mut(&mut self) -> ();

    // TODO mappings? should we take some sort of mappign summary?

    //region default impls


    //fn apply_confirmed_changes(&mut self, )

    //endregion
}

/// Modules related to genome editing
pub mod genome_editing {
    use std::collections::{HashMap, HashSet};
    use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
    use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID, FauxBrainRegionID, MaybeBrainRegionID};
    use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, FauxCorticalIdentifier};
    use crate::basis::cortical_area::cortical_identifiers::inner::random_byte_generator::CorticalIDInnerRNG;
    use crate::genome::genomes::GenomePublicInterface;

    pub(crate) struct GenomeEditCommandCollisionDetection {
        brain_regions_map: HashMap<FauxBrainRegionID, BrainRegionID>,
        removing_brain_regions: HashSet<BrainRegionID>,
        cortical_identifiers_map: HashMap<FauxCorticalIdentifier, CorticalIdentifier>,
        adding_cortical_identifiers_flat: HashSet<CorticalIdentifier>,
        removing_cortical_identifiers: HashSet<CorticalIdentifier>,
    }

    impl GenomeEditCommandCollisionDetection {

        //region Brain Region ID
        
        /// Checks if there is a parent region from a faux brain ID (IE being added within this edit set)
        pub fn get_parent_brain_region_from_real<FIQ: FeagiIndexQuantization>(
            &self,
            genuine: &BrainRegionID,
            genome: &impl GenomePublicInterface<FIQ>
        ) -> Result<BrainRegionID, GenomeEditingError> {
            if genome.contains_brain_region_id(genuine) {
                Ok(*genuine)
            } else {
                Err(GenomeEditingError::ParentBrainRegionIdNotExist)
            }
        }
        
        /// Checks if there is a parent region from a faux brain ID (IE being added within this edit set)
        pub fn get_parent_brain_region_from_faux(
            &self,
            faux_id: FauxBrainRegionID
        ) -> Result<BrainRegionID, GenomeEditingError> {
            if let Some(parent) = self.brain_regions_map.get(&faux_id) {
                Ok(parent.clone())
            } else {
                Err(GenomeEditingError::ParentBrainRegionIdNotExist)
            }
        }

        /// Checks if there is a parent region from a faux brain ID (IE being added within
        /// this edit set) or from the genome itself
        pub fn get_parent_brain_region_from_maybe<FIQ: FeagiIndexQuantization>(
            &self,
            maybe: &MaybeBrainRegionID,
            genome: &impl GenomePublicInterface<FIQ>
        ) -> Result<BrainRegionID, GenomeEditingError> {
            
            match maybe {
                MaybeBrainRegionID::Faux(f) => {
                    self.get_parent_brain_region_from_faux(f)
                }
                MaybeBrainRegionID::Genuine(g) => {
                    self.get_parent_brain_region_from_real(g, genome)
                }
            }
        }
        
        /// Converts a faux id to a real one after ensuring no collisions. If the ID bytes
        /// itself can be reusued without collision, it will do so.
        pub fn convert_id_to_add_brain_region<FIQ: FeagiIndexQuantization>(
            &mut self,
            genome: &impl GenomePublicInterface<FIQ>,
            faux: FauxBrainRegionID
        ) -> Result<BrainRegionID, GenomeEditingError> {
            
            if self.brain_regions_map.contains_key(&faux) {
                Err(GenomeEditingError::DuplicateBrainRegionIDAdded)
            } else {
                let real_id = faux.into_brain_region_id();
                if genome.contains_brain_region_id(&real_id) {
                    real_id.reroll_id();
                    if genome.contains_brain_region_id(&real_id) {
                        return Err(GenomeEditingError::BrainRegionIDAlreadyExists)
                    }
                }
                Ok(real_id)
            }
            
        }
        
        //endregion
        
        
        //region Cortical Area
        
        pub fn convert_id_to_add_area<FIQ: FeagiIndexQuantization, RNG: CorticalIDInnerRNG>(
            &mut self,
            genome: &impl GenomePublicInterface<FIQ>,
            faux: FauxCorticalIdentifier,
            rng: &RNG
        ) -> Result<CorticalIdentifier, GenomeEditingError> {
            
            if self.cortical_identifiers_map.contains_key(&faux) {
                return Err(GenomeEditingError::CorticalAreaAlreadyBeingAdded)
            }
            
            let mut cortical_id = faux.into_genuine();
            for i in 0..8 { // Did you get the broom closet ending? The broom closet ending was my favorite!
                if genome.contains_cortical_area_id(cortical_id) {
                    cortical_id.reroll_inner(rng);
                }
            };
            
            Ok(cortical_id)
        }
        
        
        //endregion

    }

    #[derive(Debug, thiserror::Error)]
    pub enum GenomeEditingError {
        #[error("Unable to add a brain region ID that already exists! Is something wrong with the RNG?")]
        BrainRegionIDAlreadyExists,
        #[error("Same Brain Region ID is being added multiple times!")]
        DuplicateBrainRegionIDAdded,
        #[error("Given Parent Brain Region ID does not exist!")]
        ParentBrainRegionIdNotExist,
        #[error("Cortical Area of the same ID is being added twice!")]
        CorticalAreaAlreadyBeingAdded,
    }

}



#[derive(Debug, thiserror::Error)]
pub enum GenomeError {
    #[error(transparent)]
    BrainRegion(#[from] BrainRegionSetError),
}




