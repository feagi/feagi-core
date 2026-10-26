use crate::basis::cortical_area::cortical_area_container::CorticalAreaRuntimeCache;


//region Connectome Data BackEnd

/// Defines how Connectome Data is stored (if its actual accessible data, or if it is a generator)
pub trait GraphConnectomeDataBackend {
    type NeuronDataType;
}

//endregion