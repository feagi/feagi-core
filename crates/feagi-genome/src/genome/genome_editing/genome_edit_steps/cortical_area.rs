use std::collections::HashSet;
use feagi_connectome::connectome_basis::connectome_editing::{ConnectomeEditResultantEnum, ConnectomeEditStepCorticalAreaAdd};
use feagi_connectome::connectome_basis::engine::BurstEngineIndex;
use feagi_connectome::connectome_basis::neuron_model::{CorticalAreaInitContainer, NeuronLayout, NeuronLayoutOf, NeuronModel, NeuronModelEnum, NeuronModelMember};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::{CorticalAreaType, CorticalAreaTypeMapMember, CorticalAreaTypeMemberAt, CorticalAreaTypeVisitor, CorticalIdentifier};
use crate::genome::containers::cortical_area::{
    CorticalAreaContainerDesigner, CorticalAreaGenerator, CorticalAreaGeneratorContainer, CorticalAreaGeneratorEnum, NeuronModelDataDesigner,
    NeuronModelDataDesignerContainer,
};
use crate::genome::genome_editing::genome_edit_steps::{GenomeEditStepType, PreparedGenomeEditStep};
use crate::genome::genome_editing::{GenomeEditError, GenomeEditSequence, GenomeEditStepResult};
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use crate::genome::genomes::designer_genome::DesignerGenome;

/// How many random cortical identifiers are tried before giving up on finding an unused one
const CORTICAL_IDENTIFIER_ATTEMPTS: usize = 64;

/// TODO used for mapping, which is not implemented yet. Until then it is not used
pub type FauxCorticalIdentifier = u64;

pub(crate) enum GenomeEditStepCorticalAreaEnum<FIQ: FeagiIndexQuantization> {
    AddArea(GenomeEditStepCorticalAreaAddArea<FIQ>),
    // TODO Remove, Edit layout, Edit properties, Edit variance
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepCorticalAreaEnum<FIQ> {
    fn genome_validity_check(&self, genome: &DesignerGenome<FIQ>) -> Result<(), GenomeEditError> {
        match self {
            GenomeEditStepCorticalAreaEnum::AddArea(step) => step.genome_validity_check(genome),
        }
    }

    fn construct_connectome_edit_step(self, genome: &DesignerGenome<FIQ>) -> Result<PreparedGenomeEditStep<FIQ>, GenomeEditError> {
        match self {
            GenomeEditStepCorticalAreaEnum::AddArea(step) => step.construct_connectome_edit_step(genome),
        }
    }
}


/// Adds a cortical area within a brain region. Its cortical type and model member are whichever
/// its generator is held for
pub(crate) struct GenomeEditStepCorticalAreaAddArea<FIQ: FeagiIndexQuantization> {
    /// TODO not used yet
    #[allow(dead_code)]
    pub faux_identifier: FauxCorticalIdentifier,
    pub generator: CorticalAreaGeneratorEnum<FIQ>,
    pub parent_region: BrainRegionID,
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepCorticalAreaAddArea<FIQ> {
    fn genome_validity_check(&self, genome: &DesignerGenome<FIQ>) -> Result<(), GenomeEditError> {
        if genome.brain_region(&self.parent_region).is_none() {
            return Err(GenomeEditError::BrainRegionNotFound(self.parent_region));
        }
        // TODO validate the layout against the limits of its cortical type
        Ok(())
    }

    fn construct_connectome_edit_step(self, genome: &DesignerGenome<FIQ>) -> Result<PreparedGenomeEditStep<FIQ>, GenomeEditError> {
        let Self { faux_identifier: _, generator, parent_region, metadata, positioning } = self;
        generator.visit(PrepareAddArea { genome, parent_region, metadata, positioning })
    }
}

/// Prepares adding a cortical area, once its cortical type and model member are known
struct PrepareAddArea<'a, FIQ: FeagiIndexQuantization> {
    genome: &'a DesignerGenome<FIQ>,
    parent_region: BrainRegionID,
    metadata: GenomeObjectMetadata,
    positioning: GenomeObjectPositioning,
}

impl<'a, FIQ: FeagiIndexQuantization> CorticalAreaTypeVisitor<CorticalAreaGeneratorContainer<FIQ>> for PrepareAddArea<'a, FIQ> {
    type Output = Result<PreparedGenomeEditStep<FIQ>, GenomeEditError>;

    fn visit<T, NM>(self, generator: CorticalAreaGenerator<FIQ, NM>) -> Self::Output
    where
        T: CorticalAreaTypeMemberAt<NM> + CorticalAreaTypeMapMember + CorticalAreaType<NeuronLayout = <NM as NeuronModel>::LayoutTag> + 'static,
        NM: NeuronModelMember + 'static,
    {
        let Self { genome, parent_region, metadata, positioning } = self;
        let inner = generate_unused_inner::<FIQ, T>(genome)?;
        let identifier = CorticalIdentifier::new::<T>(inner);

        let CorticalAreaGenerator { layout_context, universal_properties, cortical_properties, cortical_variance } = generator;
        let init = <T as CorticalAreaTypeMemberAt<NM>>::generate_init::<FIQ>(
            <NeuronLayoutOf<NM, FIQ> as NeuronLayout<FIQ>>::new(layout_context.clone()),
            universal_properties,
            cortical_properties.clone(),
            cortical_variance.clone(),
        )?;
        let connectome_edit_step = ConnectomeEditStepCorticalAreaAdd::new(NeuronModelEnum::<CorticalAreaInitContainer<FIQ>>::from_data::<NM>(init)).into();

        let apply_edit_to_genome = Box::new(move |genome: &mut DesignerGenome<FIQ>, connectome_edit_response: ConnectomeEditResultantEnum<FIQ>| {
            let ConnectomeEditResultantEnum::CorticalAreaAdd(Ok(added)) = connectome_edit_response else {
                return Err(GenomeEditError::ConnectomeEditRefused);
            };
            let neuron_model_data = NeuronModelDataDesigner::<FIQ, NM> {
                cortical_universal: universal_properties,
                cortical_properties,
                cortical_data_variance: cortical_variance,
                cortical_npu_index: added.new_cortical_index,
                // Only a single burst engine for now
                burst_engine_index: BurstEngineIndex(0),
                visualization_start_index: added.visualization_start_index,
            };
            let container = CorticalAreaContainerDesigner::<FIQ, T> {
                metadata,
                positioning,
                parent_id: parent_region,
                incoming_mappings: HashSet::new(),
                outgoing_mappings: HashSet::new(),
                layout_context,
                neuron_model_data: <T as CorticalAreaTypeMemberAt<NM>>::into_models::<NeuronModelDataDesignerContainer<FIQ>>(neuron_model_data),
            };
            let parent = genome
                .brain_regions_mut()
                .get_mut(&parent_region)
                .ok_or(GenomeEditError::BrainRegionNotFound(parent_region))?;
            parent.contained_areas_mut().insert(identifier);
            genome.cortical_areas_mut().of_mut::<T>().insert(inner, container);
            Ok(GenomeEditStepResult::CorticalAreaAdded(identifier))
        });

        Ok(PreparedGenomeEditStep { connectome_edit_step, apply_edit_to_genome })
    }
}

/// TODO placeholder until cortical types have their own identifier generators. Draws random inner
/// IDs until one is valid for cortical type `T` and not already in the genome
fn generate_unused_inner<FIQ: FeagiIndexQuantization, T: CorticalAreaTypeMapMember>(
    genome: &DesignerGenome<FIQ>,
) -> Result<T::Inner, GenomeEditError> {
    for _ in 0..CORTICAL_IDENTIFIER_ATTEMPTS {
        let mut bytes = [0u8; 6];
        rand::fill(&mut bytes[..]);
        if let Ok(inner) = T::Inner::try_from(bytes) {
            if !genome.cortical_areas().of::<T>().contains_key(&inner) {
                return Ok(inner);
            }
        }
    }
    Err(GenomeEditError::CorticalIdentifierUnavailable)
}


pub struct GenomeEditStepCorticalAreaBuilder<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomeEditSequence<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepCorticalAreaBuilder<'a, FIQ> {
    pub(crate) fn new(sequence: &'a mut GenomeEditSequence<FIQ>) -> Self {
        Self(sequence)
    }

    /// Adds a cortical area within `parent_region`, generated from `generator`, which also decides
    /// its cortical type and model member. Its identifier is returned in the result of this step
    pub fn add(
        self,
        faux_identifier: FauxCorticalIdentifier,
        generator: CorticalAreaGeneratorEnum<FIQ>,
        parent_region: BrainRegionID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        let step = GenomeEditStepCorticalAreaAddArea { faux_identifier, generator, parent_region, metadata, positioning };
        self.0.push_step(GenomeEditStepCorticalAreaEnum::AddArea(step).into());
        Ok(self.0)
    }

    // TODO Remove

    // TODO Edit Layout

    // TODO Edit properties

    // TODO Edit variance
}
