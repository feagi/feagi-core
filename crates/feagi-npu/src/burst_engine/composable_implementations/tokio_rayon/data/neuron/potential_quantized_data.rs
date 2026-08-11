use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::custom_data_types::StorageF8;
use feagi_models::wrapped_index_collections::NeuronMPIndexedVector;
use half::{bf16, f16};

pub struct NeuronQuantizedData<FIQ: FeagiIndexQuantization> {
    pub fcl_storage_f8: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<StorageF8>>,
    pub mp_storage_f8: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<StorageF8>>,
    pub fcl_f16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f16>>,
    pub mp_f16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f16>>,
    pub fcl_bf16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<bf16>>,
    pub mp_bf16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<bf16>>,
    pub fcl_f32: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f32>>,
    pub mp_f32: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f32>>,
    pub fcl_f64: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f64>>,
    pub mp_f64: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f64>>,
}

impl<FIQ: FeagiIndexQuantization> NeuronQuantizedData<FIQ> {
    pub fn new(
        fcl_storage_f8: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<StorageF8>>,
        mp_storage_f8: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<StorageF8>>,
        fcl_f16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f16>>,
        mp_f16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f16>>,
        fcl_bf16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<bf16>>,
        mp_bf16: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<bf16>>,
        fcl_f32: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f32>>,
        mp_f32: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f32>>,
        fcl_f64: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f64>>,
        mp_f64: NeuronMPIndexedVector<FIQ::NeuronIndexQuant, NeuronMembranePotential<f64>>,
    ) -> Self {
        Self {
            fcl_storage_f8,
            mp_storage_f8,
            fcl_f16,
            mp_f16,
            fcl_bf16,
            mp_bf16,
            fcl_f32,
            mp_f32,
            fcl_f64,
            mp_f64,
        }
    }
}

impl<FIQ: FeagiIndexQuantization> Default for NeuronQuantizedData<FIQ> {
    fn default() -> Self {
        Self {
            fcl_storage_f8: NeuronMPIndexedVector::new_empty(),
            mp_storage_f8: NeuronMPIndexedVector::new_empty(),
            fcl_f16: NeuronMPIndexedVector::new_empty(),
            mp_f16: NeuronMPIndexedVector::new_empty(),
            fcl_bf16: NeuronMPIndexedVector::new_empty(),
            mp_bf16: NeuronMPIndexedVector::new_empty(),
            fcl_f32: NeuronMPIndexedVector::new_empty(),
            mp_f32: NeuronMPIndexedVector::new_empty(),
            fcl_f64: NeuronMPIndexedVector::new_empty(),
            mp_f64: NeuronMPIndexedVector::new_empty(),
        }
    }
}
