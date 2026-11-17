use core::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::synapse_model::SynapseModel;

/// The axon bundle level data that should be exposed to genome developers. Not mutable during
/// axon bundle processing
pub trait AxonBundleDataPropertiesSpecification<SM: SynapseModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any axon bundle model data should be defined here (for the specification)
}

/// The axon bundle data that is for mutable internal processing, will not be
/// exposed to genome developers but is saved in the connectome
pub trait AxonBundleDataInternalSpecification<SM: SynapseModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any axon bundle model data should be defined here (for the specification)
}

/// The axon bundle data that is used for runtime processing, is not exposed to
/// genome developers nor is it saved (starts clean with every init)
pub trait AxonBundleDataScratchSpecification<SM: SynapseModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any axon bundle model data should be defined here (for the specification)
}


/// The per synapse level data that should be saved to the connectome
pub trait SynapseDataInternalSpecification<SM: SynapseModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any axon bundle model data should be defined here (for the specification)
}

/// The per synapse level data that should not be saved to the connectome
/// (starts clean with every init)
pub trait SynapseDataScratchSpecification<SM: SynapseModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any axon bundle model data should be defined here (for the specification)
}


/*

/// Data used by all Synapse Models
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynapseModelUniversalAxonBundleProperties
{

}

 */