use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::{CorticalAreaTypeEnum, CorticalAreaTypeMapValue, CorticalAreaTypeMember, CorticalAreaTypeMemberAt, CorticalIdentifier};
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use std::collections::HashSet;
use feagi_basis::quantization::quantizable::DataValueQuantizationError;
use feagi_connectome::connectome_basis::engine::BurstEngineIndex;
use feagi_connectome::connectome_basis::neuron_model::{
    NeuronLayout, NeuronLayoutOf, NeuronLayoutTag, NeuronModel, NeuronModelContainer, NeuronModelMember, NeuronModelUniversalCorticalProperties,
};
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
    pub layout_context: <<CT::NeuronLayout as NeuronLayoutTag>::Layout<FIQ> as NeuronLayout<FIQ>>::CorticalLevelContext,
    /// Whichever model this cortical type allows is in use, at whichever quantization of that model
    pub neuron_model_data: CT::Models<NeuronModelDataDesignerContainer<FIQ>>,
}


/// Holds the [`NeuronModelDataDesigner`] of a cortical area
pub struct NeuronModelDataDesignerContainer<FIQ>(core::marker::PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronModelContainer for NeuronModelDataDesignerContainer<FIQ> {
    type Of<M: NeuronModel> = NeuronModelDataDesigner<FIQ, M>;
}


/// The neuron model data of a cortical area. The universal properties, cortical properties and
/// cortical variance are what it was generated from (along with the layout of its container), so
/// it can be generated again. The rest is where it lives in the neural processor
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


/// What a new cortical area is generated from: its layout, and the parts of the defaults of its
/// cortical type a client may edit. What the cortical type fixes is filled in when generating
pub struct CorticalAreaGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    /// Details on how its neurons are laid out (dimensions, neuron count, etc)
    pub layout_context: <NeuronLayoutOf<NM, FIQ> as NeuronLayout<FIQ>>::CorticalLevelContext,
    pub universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_variance: NM::CorticalDataVariance,
}

impl<FIQ: FeagiIndexQuantization, NM: NeuronModelMember> CorticalAreaGenerator<FIQ, NM> {
    /// The defaults of cortical type `T` at this model member, with the given layout. Returns an
    /// error if a decimal default becomes infinite or NaN at this model member
    pub fn from_defaults<T: CorticalAreaTypeMemberAt<NM>>(
        layout_context: <NeuronLayoutOf<NM, FIQ> as NeuronLayout<FIQ>>::CorticalLevelContext,
    ) -> Result<Self, DataValueQuantizationError> {
        Ok(Self {
            layout_context,
            universal_properties: <T as CorticalAreaTypeMemberAt<NM>>::default_universal_properties()?,
            cortical_properties: <T as CorticalAreaTypeMemberAt<NM>>::default_cortical_properties()?,
            cortical_variance: <T as CorticalAreaTypeMemberAt<NM>>::default_cortical_variance()?,
        })
    }
}

/// Holds the [`CorticalAreaGenerator`] of a new cortical area
pub struct CorticalAreaGeneratorContainer<FIQ>(core::marker::PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronModelContainer for CorticalAreaGeneratorContainer<FIQ> {
    type Of<M: NeuronModel> = CorticalAreaGenerator<FIQ, M>;
}

/// The generator of a new cortical area, at whichever cortical type and model member it is
pub type CorticalAreaGeneratorEnum<FIQ> = CorticalAreaTypeEnum<CorticalAreaGeneratorContainer<FIQ>>;
