use feagi_connectome_neural::neural_basis::neuron_model::data::neuron_layout::implementations::voxels_default::NeuronLayoutVoxelDefault;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::{
    CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalFunctionSpecification,
    NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronModel,
};
use feagi_connectome_neural::neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use half::{bf16, f16};
use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStoreBackend;
use feagi_connectome_neural::neural_basis::neuron_model::data::data_containers::{CorticalModelFullStorageData, CorticalModelGenomePropertiesData, CorticalModelNPUInterfaceData};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use serde::{Deserialize, Serialize};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Core Power type
    CorePower

    NeuronLayout: NeuronLayoutVoxelDefault

    QuantizationLevels: {
        Float16: {
            MembranePotentialQuant: f16,
        }

        BFloat16: {
            MembranePotentialQuant: bf16,
        }

        Float32: {
            MembranePotentialQuant: f32,
        }

        Float64: {
            MembranePotentialQuant: f64,
        }
    }
}

cortical_dynamics_quantization_core_power_template!(expand_cortical_dynamics_quantization);


pub type CorePowerFullStorageData<
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower,
    SB: QuantContiguousStoreBackend
> = CorticalModelFullStorageData<FIQ, NM, SB>;

#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat16, SB>: Serialize, CorePowerFullStorageData<FIQ, NeuronModelCorePowerBFloat16, SB>: Serialize, CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat32, SB>: Serialize, CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat64, SB>: Serialize",
    deserialize = "CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat16, SB>: Deserialize<'de>, CorePowerFullStorageData<FIQ, NeuronModelCorePowerBFloat16, SB>: Deserialize<'de>, CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat32, SB>: Deserialize<'de>, CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat64, SB>: Deserialize<'de>",
))] // Serializable when the store of SB is, for every level
pub enum CorePowerFullStorageDataQuantEnum<
    FIQ: FeagiIndexQuantization,
    SB: QuantContiguousStoreBackend,
> {
    Float16(CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat16, SB>),
    BFloat16(CorePowerFullStorageData<FIQ, NeuronModelCorePowerBFloat16, SB>),
    Float32(CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat32, SB>),
    Float64(CorePowerFullStorageData<FIQ, NeuronModelCorePowerFloat64, SB>),
}

pub type CorePowerGenomePropertiesData<
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower
> = CorticalModelGenomePropertiesData<FIQ, NM>;

#[derive(Serialize, Deserialize)]
#[serde(bound = "")] // Every level is serializable through the bounds of NeuronModel and NeuronLayout
pub enum CorePowerGenomePropertiesDataEnum<
    FIQ: FeagiIndexQuantization,
> {
    Float16(CorticalModelGenomePropertiesData<FIQ, NeuronModelCorePowerFloat16>),
    BFloat16(CorticalModelGenomePropertiesData<FIQ, NeuronModelCorePowerBFloat16>),
    Float32(CorticalModelGenomePropertiesData<FIQ, NeuronModelCorePowerFloat32>),
    Float64(CorticalModelGenomePropertiesData<FIQ, NeuronModelCorePowerFloat64>),
}



pub type CorePowerNPUInterfaceData<
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelCorePower
> = CorticalModelNPUInterfaceData<FIQ, NM>;

#[derive(Serialize, Deserialize)]
#[serde(bound = "")] // Every level is serializable through the bounds of NeuronModel and NeuronLayout
pub enum CorePowerNPUInterfaceDataEnum<
    FIQ: FeagiIndexQuantization,
> {
    Float16(CorticalModelNPUInterfaceData<FIQ, NeuronModelCorePowerFloat16>),
    BFloat16(CorticalModelNPUInterfaceData<FIQ, NeuronModelCorePowerBFloat16>),
    Float32(CorticalModelNPUInterfaceData<FIQ, NeuronModelCorePowerFloat32>),
    Float64(CorticalModelNPUInterfaceData<FIQ, NeuronModelCorePowerFloat64>),
}

