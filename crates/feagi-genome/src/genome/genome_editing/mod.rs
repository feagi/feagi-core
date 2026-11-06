pub mod items;
pub mod genome_edit_step;
pub mod genome_edit_steps;

use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequesterAlloc;
use feagi_basis::parallelism::feagi_channel::request_response_channel::feagi_request_response_error::FeagiRequestResponseError;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::quantization::quantizable::DataValueQuantizationError;
use feagi_connectome::connectome_basis::connectome_editing::ConnectomeEditResultantEnum;
use feagi_connectome::connectome_basis::engine::EngineChangeCommandOtherError;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::neural_processors::singular::{NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse};
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::CorticalIdentifier;
use crate::genome::genome_editing::genome_edit_steps::brain_region::GenomeEditStepBrainRegionBuilder;
use crate::genome::genome_editing::genome_edit_steps::cortical_area::GenomeEditStepCorticalAreaBuilder;
use crate::genome::genome_editing::genome_edit_steps::{GenomeEditStepEnum, GenomeEditStepType, PreparedGenomeEditStep};
use crate::genome::genomes::designer_genome::DesignerGenome;

/// An ordered list of edits to the genome, built through its builders, then applied as a whole:
/// every step is checked against the genome first, and if any is invalid nothing is applied.
///
/// TODO steps are each checked against the genome as it was before the sequence, so a later step
/// relying on an earlier one (such as editing an area deleted earlier) is not caught. For now every
/// sequence holds a single step
pub struct GenomeEditSequence<FIQ: FeagiIndexQuantization>
{
    steps: Vec<GenomeEditStepEnum<FIQ>>,
}


impl<FIQ: FeagiIndexQuantization> GenomeEditSequence<FIQ> {
    pub fn new() -> Self {
        Self { steps: vec![] }
    }

    pub fn brain_region(&mut self) -> GenomeEditStepBrainRegionBuilder<'_, FIQ> {
        GenomeEditStepBrainRegionBuilder::new(self)
    }

    pub fn cortical_area(&mut self) -> GenomeEditStepCorticalAreaBuilder<'_, FIQ> {
        GenomeEditStepCorticalAreaBuilder::new(self)
    }

    pub fn connectivity_rule(&mut self) -> () {}

    pub fn cortical_mapping(&mut self) -> () {}

    /// How many steps are in the sequence
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub(crate) fn push_step(&mut self, genome_edit_step: GenomeEditStepEnum<FIQ>) {
        self.steps.push(genome_edit_step)
    }
}

impl<FIQ: FeagiIndexQuantization> Default for GenomeEditSequence<FIQ> {
    fn default() -> Self {
        Self::new()
    }
}

/// What a step of a genome edit sequence did, in the order of the steps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenomeEditStepResult {
    BrainRegionAdded(BrainRegionID),
    CorticalAreaAdded(CorticalIdentifier),
}

/// Why a genome edit sequence was not applied, or was not applied in full
#[derive(Debug, thiserror::Error)]
pub enum GenomeEditError {
    #[error("Brain region {0:?} does not exist")]
    BrainRegionNotFound(BrainRegionID),
    #[error("Brain region {0:?} already exists")]
    BrainRegionAlreadyExists(BrainRegionID),
    #[error("No unused cortical identifier could be found for the cortical type")]
    CorticalIdentifierUnavailable,
    #[error("A cortical area could not be generated: {0}")]
    Quantization(#[from] DataValueQuantizationError),
    #[error("The neural processor could not be reached: {0}")]
    NeuralProcessorUnreachable(#[from] FeagiRequestResponseError),
    #[error("The neural processor answered with an unexpected response")]
    NeuralProcessorUnexpectedResponse,
    #[error("The neural processor failed to apply an edit: {0}")]
    NeuralProcessorFailed(#[from] EngineChangeCommandOtherError),
    #[error("The connectome refused an edit")]
    ConnectomeEditRefused,
}

/// The command channel of the neural processor whose connectome the genome edits
pub(crate) type NeuralProcessorRequester<FIQ> = FeagiRequesterAlloc<
    FeagiWorkerRequest<NeuronProcessorSingleEngineRequest<FIQ>>,
    FeagiWorkerResponse<NeuronProcessorSingleEngineResponse<FIQ>>,
>;

/// Applies `sequence` to `genome`, and to the connectome of the neural processor if any step edits
/// it. Every step is checked first, and if any is invalid nothing is applied. If any step edits the
/// connectome, the neural processor is paused until the genome has been updated to match.
///
/// # Panics
/// If the connectome reports an edit corrupted it
pub(crate) async fn apply_genome_edit_sequence<FIQ: FeagiIndexQuantization>(
    genome: &mut DesignerGenome<FIQ>,
    neural_processor: &mut NeuralProcessorRequester<FIQ>,
    sequence: GenomeEditSequence<FIQ>,
) -> Result<Vec<GenomeEditStepResult>, GenomeEditError> {
    for step in &sequence.steps {
        step.genome_validity_check(genome)?;
    }
    let mut prepared_steps = Vec::with_capacity(sequence.steps.len());
    for step in sequence.steps {
        prepared_steps.push(step.construct_connectome_edit_step(genome)?);
    }

    // Nothing to send the neural processor, so it is left running
    if !prepared_steps.iter().any(PreparedGenomeEditStep::edits_connectome) {
        return prepared_steps
            .into_iter()
            .map(|step| (step.apply_edit_to_genome)(genome, ConnectomeEditResultantEnum::Nothing))
            .collect();
    }

    match request_neural_processor(neural_processor, NeuronProcessorSingleEngineRequest::Pause).await? {
        NeuronProcessorSingleEngineResponse::Paused => {}
        _ => return Err(GenomeEditError::NeuralProcessorUnexpectedResponse),
    }
    let applied = apply_connectome_edits(genome, neural_processor, prepared_steps).await;
    match request_neural_processor(neural_processor, NeuronProcessorSingleEngineRequest::Resume).await? {
        NeuronProcessorSingleEngineResponse::Resumed => {}
        _ => return Err(GenomeEditError::NeuralProcessorUnexpectedResponse),
    }
    applied
}

/// Sends the connectome edits to the paused neural processor, then updates the genome with what
/// each that was applied returned. The genome is updated for every edit applied before one failed,
/// so it still matches the connectome
async fn apply_connectome_edits<FIQ: FeagiIndexQuantization>(
    genome: &mut DesignerGenome<FIQ>,
    neural_processor: &mut NeuralProcessorRequester<FIQ>,
    prepared_steps: Vec<PreparedGenomeEditStep<FIQ>>,
) -> Result<Vec<GenomeEditStepResult>, GenomeEditError> {
    let (connectome_edit_steps, apply_edits_to_genome): (Vec<_>, Vec<_>) =
        prepared_steps.into_iter().map(|step| (step.connectome_edit_step, step.apply_edit_to_genome)).unzip();

    let NeuronProcessorSingleEngineResponse::ConnectomeEdits(connectome_results) =
        request_neural_processor(neural_processor, NeuronProcessorSingleEngineRequest::ConnectomeEdits(connectome_edit_steps)).await?
    else {
        return Err(GenomeEditError::NeuralProcessorUnexpectedResponse);
    };

    let mut results = Vec::with_capacity(apply_edits_to_genome.len());
    for (apply_edit_to_genome, connectome_result) in apply_edits_to_genome.into_iter().zip(connectome_results) {
        let resultant = connectome_result?;
        // TODO handle a corrupted connectome. For now nothing can be trusted, so stop here
        if resultant.is_connectome_corrupted() {
            panic!("A connectome edit reported that it corrupted the connectome");
        }
        if resultant.is_err() {
            return Err(GenomeEditError::ConnectomeEditRefused);
        }
        results.push(apply_edit_to_genome(genome, resultant)?);
    }
    Ok(results)
}

async fn request_neural_processor<FIQ: FeagiIndexQuantization>(
    neural_processor: &mut NeuralProcessorRequester<FIQ>,
    request: NeuronProcessorSingleEngineRequest<FIQ>,
) -> Result<NeuronProcessorSingleEngineResponse<FIQ>, GenomeEditError> {
    match neural_processor.make_request(FeagiWorkerRequest::Custom(request)).await? {
        FeagiWorkerResponse::Custom(response) => Ok(response),
        _ => Err(GenomeEditError::NeuralProcessorUnexpectedResponse),
    }
}
