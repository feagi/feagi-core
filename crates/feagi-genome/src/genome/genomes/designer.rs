use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_burst::burst_basis::connectome_editing::{ConnectomeEditEnum, ConnectomeEditResultantEnum};
use feagi_connectome::connectome_burst::burst_basis::multi_engine::{SingleEngineIndexing};
use feagi_connectome::connectome_burst::worker_commands::{NeuralProcessingUnitRequest, NeuralProcessingUnitResultant};
use crate::genome::editing::genome_edit_commands::{GenomeEditCommandStep, GenomeEditCommandStepEnum, GenomeEditCommands, GenomeEditRequestValidationError};
use crate::genome::editing::genome_edit_requests::load_genome::GenomeEditRequestsLoadGenome;
use crate::genome::genomes::brain_region::{BrainRegionContextDesigner, BrainRegionSet, BrainRegionSetDesigner};
use crate::genome::genomes::cortical_area::{CorticalAreaSet, CorticalAreaSetDesigner};
use crate::genome::genomes::GenomePublicInterface;

pub struct GenomeDesigner<FIQ: FeagiIndexQuantization>  {
    state: DesignerGenomeState,
    brain_regions: BrainRegionSetDesigner,
    cortical_areas: CorticalAreaSetDesigner<FIQ>,
    //neural_processor: ()
}


impl<FIQ: FeagiIndexQuantization> GenomeDesigner<FIQ>{
    
    pub async fn new_from_load_genome_sequence(
        sequence: GenomeEditRequestsLoadGenome<FIQ>,
        npu: &mut FeagiWorker

    ) -> Result<Self, GenomeEditRequestValidationError<FIQ>> {

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
            //neural_processor: npu
        };

        // Verify request, build complete command list
        let commands = GenomeEditCommands::validate_requests_into_commands(
            &steps,
            &genome
        )?;

        // build connectome requests
        let mut connectome_requests: Vec<(SingleEngineIndexing, ConnectomeEditEnum<FIQ>)> = vec![];
        for command in &commands.commands {
            match command {
                GenomeEditCommandStepEnum::AddBrainRegion(a) => {
                    let con = a.generate_connectome_edit_step();
                    connectome_requests.push((SingleEngineIndexing, con));
                }
                GenomeEditCommandStepEnum::AddCorticalArea(a) => {
                    let con = a.generate_connectome_edit_step();
                    connectome_requests.push((SingleEngineIndexing, con));
                }
            }
        }

        let npu_request = NeuralProcessingUnitRequest::ConnectomeEdits(connectome_requests);

        let npu_response = npu.make_request(npu_request).await.unwrap();

        let npu_response = match npu_response {
            NeuralProcessingUnitResultant::ConnectomeEdits(res) => res.unwrap(),
            _ => panic!("unexpected response!")
        };

        for (command, npu_res) in commands.commands.iter().zip(npu_response) {
            match command {
                GenomeEditCommandStepEnum::AddBrainRegion(a) => {
                    match npu_res {
                        ConnectomeEditResultantEnum::Nothing(_) => {
                            _ = genome.get_brain_region_set().add_new_inner_brain_region(
                                a.this_ids,
                                &a.parent_region,
                                a.metadata,
                                a.positioning
                            )
                        }
                        ConnectomeEditResultantEnum::EditConnectomeStep(_) => {
                            panic!("unexpected cnnectome brain region!")
                        }
                    }
                }
                GenomeEditCommandStepEnum::AddCorticalArea(a) => {
                    match npu_res {
                        ConnectomeEditResultantEnum::Nothing(_) => {
                            panic!("unexpected cnnectome area!")
                        }
                        ConnectomeEditResultantEnum::EditConnectomeStep(e) => {
                            genome.cortical_areas.add_new_cortical_area(
                                // TODO
                            )
                        }
                    }
                }
            }
        }



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




