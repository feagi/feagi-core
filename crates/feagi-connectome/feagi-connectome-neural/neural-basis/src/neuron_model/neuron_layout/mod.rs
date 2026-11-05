//! Defines how neurons are laid out within a cortical area, in what pattern

use core::fmt::Debug;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use crate::quantization_levels::FeagiIndexQuantization;

pub mod voxel;
pub mod single_stepped_dynamic;


/// Root trait for defining Neuron Layout, the struct that defines how the neurons are laid out
/// in a cortical area
pub trait NeuronLayout<
    FIQ: FeagiIndexQuantization,
>: Clone + Copy + Debug + PartialEq + Sync + Send
+ serde::Serialize + serde::de::DeserializeOwned + Sized + 'static
{
    /// How and if the neuron layout can change on its own (cortical area grows)
    const CHANGE_POLICY: NeuronLayoutChangePolicy;

    /// If the neurons is stored in a dense array or some sparse structure
    const SPARSITY: NeuronLayoutSparsity;

    /// What data describes the cortical context that summerizes the state of the whole area
    type CorticalLevelContext: Clone;

    /// The per neuron context, helps identify a neurons "location" relative to others within a
    /// cortical area
    type NeuronLevelContext;


    fn validate_with_restrictor_and_new<Res>(init_data: Self::CorticalLevelContext, restrictor: &Res)
        -> Result<Self, NeuronLayoutRestrictionError>
    where
        Res: NeuronLayoutRestrictor<FIQ, CorticalLevelContext=Self::CorticalLevelContext>
    {
        restrictor.validate_cortical_context(&init_data)?;
        Ok(Self::new(init_data))
    }

    /// Do not call directly! Use `validate_and_new`!
    /// Init the cortical context after validation
    fn new(init_data: Self::CorticalLevelContext) -> Self;

    /// Gets the (max) number of neurons that this layout encloses to have allocation for
    fn get_number_neurons_to_allocate_for(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;

    /// Gets the struct denoting the "location" of a neuron in the context of the cortical area
    fn get_neuron_layout_context(
        &self,
        neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> Self::NeuronLevelContext;

    // NOTE: No Resizing directly. If we need to resize the cortical area, we init a new one and
    // copy over what is needed
}

/// Allows defining the Neuron Layout type without knowing the FIQ in the moment
pub trait NeuronLayoutTag {
    type Layout<FIQ: FeagiIndexQuantization>: NeuronLayout<FIQ>;
}


/// May impose restrictions on the cortical level context
pub trait NeuronLayoutRestrictor<FIQ: FeagiIndexQuantization>: Clone + Copy + Debug + PartialEq
+ Sync + Send + serde::Serialize + serde::de::DeserializeOwned + Sized + 'static
{
    /// What data describes the cortical context that summerizes the state of the whole area
    type CorticalLevelContext;


    fn validate_cortical_context(&self, context: &Self::CorticalLevelContext) -> Result<(), NeuronLayoutRestrictionError>;
}

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

#[derive(Debug, thiserror::Error)]
pub enum NeuronLayoutRestrictionError {
    #[error("Given Neuron layout would exceed current index quantization limits!")]
    ViolatesQuantizationMax,
    #[error("Invalid Voxel Dimensions")]
    Voxel(#[from] voxel::NeuronLayoutRestrictionVoxelError),

    // nothing for SingleSteppedDynamicSparse

}