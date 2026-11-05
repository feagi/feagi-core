pub mod model_data;
pub mod voxel_to_voxel;

use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::synapse_model::model_data::{AxonBundleDataInternalSpecification, AxonBundleDataPropertiesSpecification, AxonBundleDataScratchSpecification, SynapseDataInternalSpecification, SynapseDataScratchSpecification};

pub trait SynapseModel: MembranePotentialQuantization {

    /// What type of layout the source area is allowed to have
    type SourceAreaLayoutTagRestriction: NeuronLayoutTag;

    /// What type of layout the destination area is allowed to have
    type DestinationAreaLayoutTagRestriction: NeuronLayoutTag;

    /// The axon bundle level data that should be exposed to genome developers. Not mutable during
    /// axon bundle  processing
    type AxonBundleDataProperties: AxonBundleDataPropertiesSpecification<Self>;
    /// The axon bundle level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type AxonBundleDataInternal: AxonBundleDataInternalSpecification<Self>;
    /// The axon bundle level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type AxonBundleDataScratch: AxonBundleDataScratchSpecification<Self>;

    /// The per synapse level data that should be saved to the connectome
    type SynapseDataInternal: SynapseDataInternalSpecification<Self>;
    /// The per synapse level data that should not be saved to the connectome
    /// (starts clean with every init)
    type SynapseDataScratch: SynapseDataScratchSpecification<Self>;
}

/// Names one synapse model as a whole, across its quantization levels
pub trait SynapseModelTag {
    
}

/// This marker trait is used to implement per synapse model dynamics per burst engine
/// backend. It is to be extended with a synapse subtype trait due to the radical difference
/// in synapse models
pub trait SynapseBurstEngineDynamics<FIQ, SM>
where 
    FIQ: FeagiIndexQuantization,
    SM: SynapseModel
{}


pub mod wrapped_indexing {
    use feagi_basis::create_wrapped_quantized_unsigned_integer;

    create_wrapped_quantized_unsigned_integer!(
        /// Represents the index of an axon bundle within a burst engine
        pub AxonBundleEngineIndex
    );

    create_wrapped_quantized_unsigned_integer!(
        /// Represents the index of an axon bundle within a model and quantization
        pub AxonBundleQuantIndex
    );

    create_wrapped_quantized_unsigned_integer!(
        /// Represents the index of a synapse within a burst engine
        pub SynapseEngineIndex
    );

    create_wrapped_quantized_unsigned_integer!(
        /// Represents the index of a synapse within a model and quantization
        pub SynapseQuantIndex
    );
}
