use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::genome::genome_editing::edit_context::load_genome::GenomeEditSequenceContextLoadGenome;
use crate::genome::genome_editing::GenomeEditSequence;
use crate::genome::genomes::brain_region::{BrainRegionSet, BrainRegionSetDesigner};
use crate::genome::genomes::cortical_area::{CorticalAreaSet, CorticalAreaSetDesigner};


pub struct GenomeDesigner<FIQ: FeagiIndexQuantization>  {
    state: DesignerGenomeState,
    brain_regions: BrainRegionSetDesigner,
    cortical_areas: CorticalAreaSetDesigner<FIQ>,
    neural_processor: ()
}


impl<FIQ: FeagiIndexQuantization> GenomeDesigner<FIQ>{
    
    pub fn new_from_load_genome_sequence(sequence: GenomeEditSequence<FIQ, GenomeEditSequenceContextLoadGenome>, npu: ()) -> Result<Self, ()> {

        let GenomeEditSequence {
            context,
            steps
        } = sequence;

        let GenomeEditSequenceContextLoadGenome {
            genome_metadata,
            target_burst_rate,
            new_root_ids
        } = context;

        let mut genome = GenomeDesigner {
            state: DesignerGenomeState::Paused,
            brain_regions: BrainRegionSetDesigner::new_brain_region_set(genome_metadata, new_root_ids),
            cortical_areas: CorticalAreaSetDesigner::new(),
            neural_processor: npu
        };

        // TODO validate_genome_sequence_steps_and_build_connectome_steps
        // TODO if connectome steps, interupt connectome, send sequence, get result
        // TODO apply, resume



        
        
        Ok(genome)
    }

    fn validate_genome_sequence_steps_and_build_connectome_steps(steps: ()) -> Result<(), ()> {
        Ok(())
    }
    
    async fn send_verified_request_to_npu() -> Result<(), ()> {
        Ok(())
    }
}



/// Akin to NPU state, but we keep this local for rapid lookups and responses
pub enum DesignerGenomeState {
    Paused,
    WaitingOnNPU,
    Running(TargetFrequency)
}




