// TODO build.rs should build this struct tree

use crate::neuron::genome_interface::cortical_area_spawner::DimensionalCorticalAreaSpawner;
use crate::neuron::models::feagi_advanced::{
    FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData, FeagiAdvancedModelQuantization, FeagiAdvancedModelStandardQuant,
};
use core::marker::PhantomData;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensions;
use feagi_data::neurons::NeuronVoxelDensityIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Requests regarding cortical areas
pub struct NPURequestCorticalArea<FIQ: FeagiIndexQuantization> {
    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NPURequestCorticalArea<FIQ> {
    #[doc(hidden)]
    /// Creates a cortical area request. Generally should not be used directly.
    /// Use `NPURequestBuilder` instead.
    pub fn create_npu_cortical_request() -> Self {
        NPURequestCorticalArea { _p: PhantomData }
    }

    pub fn feagi_advanced() -> NPURequestCorticalAreaFeagiAdvanced<FIQ> {
        NPURequestCorticalAreaFeagiAdvanced { _p: PhantomData }
    }
}

pub struct NPURequestCorticalAreaFeagiAdvanced<FIQ: FeagiIndexQuantization> {
    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NPURequestCorticalAreaFeagiAdvanced<FIQ> {
    pub fn standard() -> NPURequestCorticalAreaFeagiAdvancedQuant<FIQ, FeagiAdvancedModelStandardQuant> {
        NPURequestCorticalAreaFeagiAdvancedQuant { _p: PhantomData }
    }
}

pub struct NPURequestCorticalAreaFeagiAdvancedQuant<FIQ: FeagiIndexQuantization, NMQ: FeagiAdvancedModelQuantization> {
    _p: PhantomData<(FIQ, NMQ)>,
}

impl<FIQ: FeagiIndexQuantization, NMQ: FeagiAdvancedModelQuantization> NPURequestCorticalAreaFeagiAdvancedQuant<FIQ, NMQ> {
    // TODO define cortical class
    pub fn create_dimensional_cortical_area(dimensions: NeuronVoxelDimensions<u32>, neurons_per_voxel: NeuronVoxelDensityIndex<u32>) {
        todo!()
    }

    // TODO Edit

    // TODO Delete

    // TODO Resize

    // TODO Change Quantization

    // TODO Change Neuron Model
}

pub struct DimensionalCorticalAreaSpawnerContainer {
    request: DimensionalCorticalAreaSpawnerEnum,
    dimensions_u64: (),
}

#[allow(non_camel_case_types)]
pub(crate) enum DimensionalCorticalAreaSpawnerEnum {
    FeagiAdvanced_Standard(
        Box<
            dyn DimensionalCorticalAreaSpawner<
                FeagiAdvancedModelStandardQuant,
                FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandardQuant>,
                FeagiAdvancedModelNeuronData<FeagiAdvancedModelStandardQuant>,
            >,
        >,
    ),
}
