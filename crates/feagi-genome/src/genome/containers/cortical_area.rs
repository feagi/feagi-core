use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::{CorticalAreaTypeMapValue, CorticalAreaTypeMember, CorticalIdentifier, NeuronLayoutTag};
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome::cortical_area::{CorticalAreaEngineIndex, NeuronModelUniversalCorticalProperties};
use feagi_connectome::neuron_models::family::NeuronModelContainer;
use feagi_connectome::quantization_levels::FeagiIndexQuantization;
use neural_basis::neuron_model::NeuronModel;
use std::collections::HashSet;

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
    pub layout_context: <<CT::NeuronLayout as NeuronLayoutTag>::Layout<FIQ> as NeuronLayout<FIQ>>::CorticalLevelContext,
    /// Whichever model this cortical type allows is in use, at whichever quantization of that model
    pub neuron_model_data: CT::Models<NeuronModelDataDesignerContainer<FIQ>>,
}

impl<FIQ, CT> CorticalAreaContainerDesigner<FIQ, CT>
where
    FIQ: FeagiIndexQuantization,
    CT: CorticalAreaTypeMember
{



}


/// Holds the [`NeuronModelDataDesigner`] of a cortical area
pub struct NeuronModelDataDesignerContainer<FIQ>(core::marker::PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronModelContainer for NeuronModelDataDesignerContainer<FIQ> {
    type Of<M: NeuronModel> = NeuronModelDataDesigner<FIQ, M>;
}


pub struct NeuronModelDataDesigner<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub cortical_universal: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_data_variance: NM::CorticalDataVariance,
    pub cortical_npu_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub burst_engine_index: BurstEngineIndex,
    // TODO NM specific visualization context
    // TODO generator used as an enum
}





/*

pub struct CorticalAreaContainerTypedStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel>
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub cortical_area_model: (),
}

 */



