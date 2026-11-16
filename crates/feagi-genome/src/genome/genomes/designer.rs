use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::{BrainRegionIdAndAncestryConstructor};
use crate::basis::cortical_area::CorticalTypedGenerator;
use crate::genome::genome_editing::edit_context::load_genome::GenomeEditSequenceContextLoadGenome;
use crate::genome::genome_editing::GenomeEditSequence;
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genomes::brain_region::{BrainRegionSet, BrainRegionSetDesigner};
use crate::genome::genomes::cortical_area::{CorticalAreaSet, GenomeCorticalAreaContextDesigner};


pub struct GenomeDesigner<FIQ: FeagiIndexQuantization>  {
    state: DesignerGenomeState,
    brain_regions: BrainRegionSetDesigner,
    cortical_areas: CorticalAreaSet<
        FIQ,
        GenomeCorticalAreaContextDesigner<FIQ>,
        CorticalTypedGenerator<FIQ>
    >
}


impl<FIQ: FeagiIndexQuantization> GenomeDesigner<FIQ>{
    
    pub fn new_from_load_genome_sequence(sequence: GenomeEditSequence<FIQ, GenomeEditSequenceContextLoadGenome>) {
        
    }
    
    
    /// Called on init, and can be called to have everything wiped. Loads with a genome with
    /// only the root area
    pub fn new_with_void_genome(
        genome_metadata: GenomeObjectMetadata,
        root_ids: BrainRegionIdAndAncestryConstructor
    ) -> Self {

        let brain_region_set = BrainRegionSet::new_brain_region_set(
            genome_metadata,
            root_ids
        );

        let cortical_area_set = CorticalAreaSet::new();

        Self {
            state: DesignerGenomeState::WaitingOnNPU,
            brain_regions: brain_region_set,
            cortical_areas: cortical_area_set,
        }
    }

    // TODO new from sequence

}



/// Akin to NPU state, but we keep this local for rapid lookups and responses
pub enum DesignerGenomeState {
    Paused,
    WaitingOnNPU,
    Running(TargetFrequency)
}




