use serde::{Deserialize, Serialize};
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::BrainRegionIdAndAncestryConstructor;
use crate::genome::editing::genome_edit_requests::{GenomeEditRequests, GenomeEditSequenceIntensity};
use crate::genome::editing::genome_edit_requests::steps::GenomeEditStepEnumAdditive;
use crate::genome::genome_object::GenomeObjectMetadata;


/// A Genome Edit Sequence specific for loading a new genome. Will cause a genome reload.
/// Can be sent empty without further steps to load a void genome. Steps can only add data, not
/// edit or remove anything
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct GenomeEditRequestsLoadGenome<FIQ: FeagiIndexQuantization> {
    genome_metadata: GenomeObjectMetadata,
    target_burst_rate: TargetFrequency,
    new_root_ids: BrainRegionIdAndAncestryConstructor,
    steps: Vec<GenomeEditStepEnumAdditive<FIQ>>
}

impl<FIQ: FeagiIndexQuantization> GenomeEditRequestsLoadGenome<FIQ> {

    //region Default Genome Reloads

    // TODO potentially make this one private, we may not want to allow building void genomes but this is a good start point
    /// Builds a new request to reset to a void empty genome, with nothing but the root
    /// brain region. No Cortical areas, No Mappings, No Connectivity Rules
    pub fn request_reload_to_void_genome(
        genome_metadata: GenomeObjectMetadata,
        target_burst_rate: TargetFrequency,
        new_root_ids: BrainRegionIdAndAncestryConstructor,
    ) -> Self {
        Self {
            genome_metadata,
            target_burst_rate,
            new_root_ids,
            steps: vec![],
        }
    }

    // TODO barebones, other setups?

    //endregion

    pub fn genome_metadata(&self) -> &GenomeObjectMetadata {
        &self.genome_metadata
    }

    pub fn target_burst_rate(&self) -> TargetFrequency {
        self.target_burst_rate
    }

    pub fn new_root_ids(&self) -> &BrainRegionIdAndAncestryConstructor {
        &self.new_root_ids
    }
}

impl<FIQ: FeagiIndexQuantization> GenomeEditRequests<FIQ> for GenomeEditRequestsLoadGenome<FIQ> {
    type EditStepEnum = GenomeEditStepEnumAdditive<FIQ>;

    fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        GenomeEditSequenceIntensity::ConnectomeReloading // This always reloads
    }

    fn add_step(&mut self, step: impl Into<Self::EditStepEnum>) {
        self.steps.push(step.into())
    }

    fn iter_steps(&self) -> impl Iterator<Item = &Self::EditStepEnum> {
        self.steps.iter()
    }
}