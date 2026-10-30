use crate::cortical_area_types::enums::CorticalAreaTypeEnum;
use feagi_connectome::cortical_area::neuron_layout::implementations::linear::NeuronLayoutLinearType;
use feagi_connectome::cortical_area::{CorticalGeneratorDefinition, NeuronLayoutType, NeuronModel};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
// TODO NeuronLayoutRestrictor to enable sensor / motors with various size limits without
// needed to make a type for each! (or part of NM?)

/// Describes a specific cortical area type. A cortical type type could use various models
/// (and their quantizations) but it must stick to a certain neuron layout
pub trait CorticalAreaType<NM>:
where
    NM: NeuronModel, // May be more restricted depending on type
{
    /// This type as an enum
    const CORTICAL_AREA_TYPE: CorticalAreaTypeEnum;
    
    /// What type of neuron layout this type uses. May be rather generic or more restricted
    type NeuronLayout: NeuronLayoutType;

    // TODO NeuronLayoutRestrictor
    
    type CorticalIDGenerator;

    
    type CorticalAreaGenerator<FIQ: FeagiIndexQuantization>: CorticalGeneratorDefinition<FIQ, NM>;
}


/// Describes a cortical area type of subtype core. Core subtype areas have unique properties
/// within the genome
pub trait CoreCorticalAreaSubType<NM>: CorticalAreaType<NM>
where
    NM: NeuronModel, // Note that many core area types may have their own unique neuron model
{
    // TODO stuff
}

/// Describes a cortical area type of subtype memory. Memory subtype areas store information
/// and can grow dynamically. They are trained.
pub trait MemoryCorticalAreaSubType<NM>: CorticalAreaType<NM>
where
    NM: NeuronModel<LayoutType=NeuronLayoutLinearType>, // Memory models must be linear (no dimensions)
{
    // TODO stuff
}