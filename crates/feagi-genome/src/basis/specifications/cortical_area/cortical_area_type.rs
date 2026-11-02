use crate::basis::cortical_area_types::cortical_area_type::CorticalAreaTypeKind;
use feagi_connectome::cortical_area::{CorticalAreaGenerator, NeuronLayoutType, NeuronModel};
use feagi_connectome::cortical_area::neuron_layout::implementations::linear::NeuronLayoutLinearType;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
// TODO NeuronLayoutRestrictor to enable sensor / motors with various size limits without
// needed to make a type for each! (or part of NM?)

/// Describes a specific cortical area type. A cortical type type could use various models
/// (and their quantizations) but it must stick to a certain neuron layout
pub trait CorticalAreaType<NM>:
where
    NM: NeuronModel, // May be more restricted depending on type
{
    /// This type as a kind
    const CORTICAL_AREA_TYPE: CorticalAreaTypeKind;

    /// What type of neuron layout this type uses. May be rather generic or more restricted
    type NeuronLayout: NeuronLayoutType;

    // TODO NeuronLayoutRestrictor

    type CorticalIDGenerator; // TODO


    type CorticalAreaGenerator<FIQ: FeagiIndexQuantization>: CorticalAreaGenerator<FIQ, NM>;
}


/// Describes a cortical area type of super type foundation. Foundation super type areas have
/// unique properties within the genome
pub trait FoundationCorticalAreaSubType<NM>: CorticalAreaType<NM>
where
    NM: NeuronModel, // Note that many foundation area types may have their own unique neuron model
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
