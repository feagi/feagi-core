use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::genome::editing::genome_edit_commands::{GenomeEditCommands, GenomeEditRequestValidationError};
use crate::genome::editing::genome_edit_requests::load_genome::GenomeEditRequestsLoadGenome;
use crate::genome::genomes::brain_region::{BrainRegionContextDesigner, BrainRegionSet, BrainRegionSetDesigner};
use crate::genome::genomes::cortical_area::{CorticalAreaSet, CorticalAreaSetDesigner};
use crate::genome::genomes::GenomePublicInterface;

pub struct GenomeDesigner<FIQ: FeagiIndexQuantization>  {
    state: DesignerGenomeState,
    brain_regions: BrainRegionSetDesigner,
    cortical_areas: CorticalAreaSetDesigner<FIQ>,
    neural_processor: ()
}


impl<FIQ: FeagiIndexQuantization> GenomeDesigner<FIQ>{
    
    pub fn new_from_load_genome_sequence(sequence: GenomeEditRequestsLoadGenome<FIQ>, npu: ()) -> Result<Self, GenomeEditRequestValidationError<FIQ>> {
        
        let GenomeEditRequestsLoadGenome {
            genome_metadata,
            target_burst_rate,
            new_root_ids,
            steps
        } = sequence;

        let mut genome = GenomeDesigner {
            state: DesignerGenomeState::Paused,
            brain_regions: BrainRegionSetDesigner::new_brain_region_set(genome_metadata, new_root_ids),
            cortical_areas: CorticalAreaSetDesigner::new(),
            neural_processor: npu
        };

        // Verify request, build complete command list
        let commands = GenomeEditCommands::validate_requests_into_commands(
            &steps,
            &genome
        )?;
        
        
        
        // TODO NPU communication

        
        
        
        Ok(genome)
    }


    
    async fn send_verified_request_to_npu() -> Result<(), ()> {
        Ok(())
    }
}

impl<FIQ: FeagiIndexQuantization> GenomePublicInterface<FIQ> for GenomeDesigner<FIQ> {
    type BrainRegionSet = BrainRegionSetDesigner;
    type BrainRegionContext = BrainRegionContextDesigner;
    type CorticalAreaSet = CorticalAreaSetDesigner<FIQ>;

    fn get_brain_region_set(&self) -> &Self::BrainRegionSet {
        &self.brain_regions
    }

    fn get_cortical_area_set(&self) -> &Self::CorticalAreaSet {
        &self.cortical_areas
    }
}



/// Akin to NPU state, but we keep this local for rapid lookups and responses
pub enum DesignerGenomeState {
    Paused,
    WaitingOnNPU,
    Running(TargetFrequency)
}




