use feagi_structures::feagi_data::feagi_ecs::collection::FeagiECSCollectionDataLivesOnCPU;
use feagi_structures::feagi_data::quantizable_spatial::index::{SpatialIndexDimensions4D};
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};

//region Base "Tag" Traits

#[doc(hidden)]
/// Root trait for all cortical data implementations
pub trait NeuronModelCorticalDataCommonDevice<CAIQ, NMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // nothing
}


/// Base trait for all cortical data for linear cortical areas
pub trait NeuronModelCorticalDataLinearDevice<CAIQ, NMQ>:
NeuronModelCorticalDataCommonDevice<CAIQ, NMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // on new device extension, if linear, extend with linear neuron model cortical area parameters
}

/// Base trait for all cortical data with dimensions (voxels)
pub trait NeuronModelCorticalDataDimensionalDevice<CAIQ, NMQ>:
NeuronModelCorticalDataCommonDevice<CAIQ, NMQ> // needs to extend due to other trait usage
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // on new device extension, if dimensional, extend with linear neuron model cortical area parameters
}

//endregion

// region CPU

#[doc(hidden)]
/// Root trait for all cortical data implementations
pub trait NeuronModelCorticalDataCommonCPU<CAIQ, NMQ>:
NeuronModelCorticalDataCommonDevice<CAIQ, NMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // nothing
}

/// Base trait for cortical data with linear neuron data that lives on the cpu
pub trait NeuronModelCorticalDataLinearCPUTemplate<CAIQ, NMQ>:
NeuronModelCorticalDataCommonCPU<CAIQ, NMQ>
+ NeuronModelCorticalDataLinearDevice<CAIQ, NMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // if linear, extend with linear neuron model cortical area parameters
}

/// Base trait for cortical data with dimensional neuron data that lives on the cpu
pub trait NeuronModelCorticalDataDimensionalCPUTemplate<CAIQ, NMQ>:
NeuronModelCorticalDataCommonCPU<CAIQ, NMQ>
+ NeuronModelCorticalDataDimensionalDevice<CAIQ, NMQ>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
{
    // if dimensional, extend with linear neuron model cortical area parameters
}

//endregion


