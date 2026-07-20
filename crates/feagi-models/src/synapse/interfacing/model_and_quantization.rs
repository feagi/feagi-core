use crate::synapse::models::uniform_weight::quantization::UniformSynapseModelQuantizationLevel;

// TODO build.rs should generate these enums


#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SynapseModelType {
    UniformWeight
}




/// Using a nested enum, easily describes the synapse model and the synapse model quantization it uses
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SynapseModelTypeAndQuantization {
    UniformWeight(UniformSynapseModelQuantizationLevel),
}


#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Default, Hash, PartialEq, Eq)]
pub enum PackedSynapseModelTypeAndQuantization {
    #[default]
    UniformWeight_Standard32Bit,
}
