use crate::basis::brain_region_identifiers::BrainRegionID;
use crate::basis::cortical_area::{CorticalAreaTypeEnum, CorticalAreaTypeMapValue, CorticalAreaTypeMember, CorticalIdentifier};
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use std::collections::HashSet;
use feagi_connectome::connectome_basis::engine::BurstEngineIndex;
use feagi_connectome::connectome_basis::neuron_model::{NeurogenesisDefaultsGenerator, NeuronLayoutData, NeuronLayoutTag, NeuronModel, NMContainer, NeuronModelUniversalCorticalProperties};
use feagi_connectome::connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_basis::visualization::NeuronVisualizationWordIndex;

/// What the designer genome holds for each cortical area, whichever its cortical type
pub struct CorticalAreaContainersDesigner<FIQ>(core::marker::PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> CorticalAreaTypeMapValue for CorticalAreaContainersDesigner<FIQ> {
    type Of<T: CorticalAreaTypeMember> = CorticalAreaContainerDesigner<FIQ, T>;
}


pub struct CorticalAreaContainerDesigner<FIQ, CT>
where
    FIQ: FeagiIndexQuantization,
    CT: CorticalAreaTypeMember
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
    pub layout_context: <<CT::NeuronLayout as NeuronLayoutTag>::Layout<FIQ> as NeuronLayoutData<FIQ>>::CorticalLevelContext, // dimensions or neuron count or other
    /// Whichever model this cortical type allows is in use, at whichever quantization of that model
    pub neuron_model_data: CT::Models<NeuronModelDataDesignerContainer<FIQ>>,
}



/// Holds the [`NeuronModelDataDesigner`] of a cortical area
pub struct NeuronModelDataDesignerContainer<FIQ>(core::marker::PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NeuronModelDataDesignerContainer<FIQ> {
    type Of<M: NeuronModel> = NeuronModelDataDesigner<FIQ, M>;
}

/// The neuron model data of a cortical area
pub struct NeuronModelDataDesigner<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub cortical_universal: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_data_variance: NM::CorticalDataVariance,
    pub cortical_npu_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub burst_engine_index: BurstEngineIndex,
    /// Where the visualization of this cortical area starts
    pub visualization_start_index: NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>,

    // TODO NM specific visualization context


}



/// Holds what a new cortical area is generated from: a [`NeurogenesisDefaultsGenerator`] with the
/// defaults of its cortical type, which a client may edit before the area is created. Get one from
/// the generator wrapper of the cortical type, or generically through
/// `CorticalAreaTypeMemberAt::new_defaults_generator`
pub struct NeurogenesisDefaultsGeneratorContainer<FIQ>(core::marker::PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NMContainer for NeurogenesisDefaultsGeneratorContainer<FIQ> {
    type Of<M: NeuronModel> = NeurogenesisDefaultsGenerator<FIQ, M>;
}

/// The generator of a new cortical area, at whichever cortical type and model member it is
pub type NeurogenesisDefaultsGeneratorCorticalTypeEnum<FIQ> = CorticalAreaTypeEnum<NeurogenesisDefaultsGeneratorContainer<FIQ>>;




/*

pub struct CorticalAreaContainerTypedStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel>
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub cortical_area_model: (),
}

 */



