use half::{bf16, f16};
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::custom_data_types::StorageF8;
use feagi_models::wrapped_index_collections::NeuronMPIndexedVector;

pub struct NeuronQuantizedData<FIQ: FeagiIndexQuantization> {
    pub fcl_storage_f8: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<StorageF8>>,
    pub mp_storage_f8: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<StorageF8>>,
    pub fcl_f16: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<f16>>,
    pub mp_f16: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<f16>>,
    pub fcl_bf16: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<bf16>>,
    pub mp_bf16: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<bf16>>,
    pub fcl_f32: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<f32>>,
    pub mp_f32: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<f32>>,
    pub fcl_f64: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<f64>>,
    pub mp_f64: NeuronMPIndexedVector<FIQ::NeuronIndexCountQuant, NeuronMembranePotential<f64>>,
}