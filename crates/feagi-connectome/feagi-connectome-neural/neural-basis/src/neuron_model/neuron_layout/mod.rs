//! Defines how neurons are laid out within a cortical area, in what pattern

use core::fmt::Debug;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use crate::neuron_model::neuron_layout::voxel::NeuronLayoutRestrictionVoxelError;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

pub mod voxel;
pub mod single_stepped_dynamic;


/// Root trait for defining Neuron Layout, the struct that defines how the neurons are laid out
/// in a cortical area. Can Only be constructed through a `NeuronLayoutRestrictor`
pub trait NeuronLayout<FIQ: FeagiIndexQuantization, >: Clone + Copy + Debug + PartialEq + Sync + Send
+ serde::Serialize + serde::de::DeserializeOwned + Sized + 'static
{
    // Can only be constructed by `NeuronLayoutRestrictor`

    /// How and if the neuron layout can change on its own (cortical area grows)
    const CHANGE_POLICY: NeuronLayoutChangePolicy;

    /// If the neurons is stored in a dense array or some sparse structure
    const SPARSITY: NeuronLayoutSparsity;

    /// What data describes the cortical context that summerizes the state of the whole area
    type CorticalLevelContext: Clone + Debug + PartialEq + serde::Serialize + serde::de::DeserializeOwned;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type NeuronLevelContext;

    /// Get the saved cortical area layout context (ex: dimensions, size, etc)
    fn get_cortical_layout_context(&self) -> &Self::CorticalLevelContext;

    /// Gets the (max) number of neurons that this layout encloses to have allocation for
    fn get_number_neurons_to_allocate_for(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;

    /// Gets the struct denoting the "location" of a neuron in the context of the cortical area
    fn get_neuron_layout_context(
        &self,
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> Result<Self::NeuronLevelContext, NeuronLayoutError>; // TODO unchecked version
}


/// May impose restrictions on the cortical level context. Also responsible for spawning NeuronLayouts
pub trait NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>: Clone + Copy + Debug + PartialEq
+ Sync + Send + serde::Serialize + serde::de::DeserializeOwned + Sized + 'static
{
    /// The type of layout this supports
    type NeuronLayout: NeuronLayout<FIQ>;

    /// Simply validates the cortical level context if its valid
    fn validate_cortical_context(
        context: &<Self::NeuronLayout as NeuronLayout<FIQ>>::CorticalLevelContext
    ) -> Result<(), NeuronLayoutError>;

    /// Validates a cortical level context, then returns the Neuron Layout if its valid
    fn validate_and_create_neuron_layout(
        context: <Self::NeuronLayout as NeuronLayout<FIQ>>::CorticalLevelContext
    ) -> Result<Self::NeuronLayout, NeuronLayoutError>;

}



/// Allows defining the Neuron Layout type without knowing the FIQ in the moment
pub trait NeuronLayoutTag: Clone + Debug {
    type Layout<FIQ: FeagiIndexQuantization>: NeuronLayout<FIQ>;
}

/// Returns the Neuron Layout of the given Neuron Model at the given index quantization
pub type NeuronLayoutOf<FIQ, NM> = <<NM as NeuronModel>::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>;

/// Returns the Cortical Context of the Neuron layout of the given Neuron model at the given index Quant
pub type CorticalLayoutContextOf<FIQ, NM>
    = <<<NM as NeuronModel>::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ> as NeuronLayout<FIQ>>::CorticalLevelContext;



/// Defines how a cortical area of a Neuron Layout may change size on its own. Allows the burst
/// engine to make various considerations
pub enum NeuronLayoutChangePolicy {
    /// The layout will not change size on its own
    Fixed,
    /// The layout may grow, but only by a single neuron per burst
    GrowthBySingleNeuron,
}

/// Is the neuron data dense (linear) or sparse
pub enum NeuronLayoutSparsity {
    Dense,
    Sparse,
}

#[derive(Debug, thiserror::Error, Clone, Hash, Copy)]
pub enum NeuronLayoutError {
    #[error("Given Neuron layout would exceed current index quantization limits!")]
    ViolatesQuantizationMax,
    #[error("Neuron Index out of Bounds!")]
    NeuronIndexOutOfBounds,
    #[error("Cortical Layout must include at least one neuron!")]
    EmptyNeuronLayout,
    #[error("Given layout fails dimensional restriction!")]
    VoxelRestriction(#[from] NeuronLayoutRestrictionVoxelError),
}

