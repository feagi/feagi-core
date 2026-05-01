use feagi_structures::base_quantizable::QuantizationLevel;
use feagi_structures::genomic::cortical_area::CorticalAreaModelType;
// NOTE: Due to quantization level taking 2 bits automatically, we have 6 bits for the cortical,
// meaning within a u8 we can identify 64 different types of cortical areas

/// Optimally allows categorizing different types of cortical areas and their quantization levels
/// in the space of a single byte
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum NPUCorticalAreaIdentifierFlag {
    CoreFeagiStandard(QuantizationLevel),
    SensoryFeagiStandard(QuantizationLevel),
    MotorFeagiStandard(QuantizationLevel),
    InterNeuronFeagiStandard(QuantizationLevel),
    InterNeuronLIF(QuantizationLevel),
    InterNeuronIzhikevich(QuantizationLevel),
    InterNeuronHodgkinHuxley(QuantizationLevel),
    Memory(QuantizationLevel),
}

impl NPUCorticalAreaIdentifierFlag {
    
    /// For interneurons, allows creation from a model type and quantization level
    pub(crate) const fn from_quantization_and_model(quantization_level: QuantizationLevel, model_type: CorticalAreaModelType) -> Self {
        match model_type {
            CorticalAreaModelType::FeagiStandard => {NPUCorticalAreaIdentifierFlag::InterNeuronFeagiStandard(quantization_level)}
            CorticalAreaModelType::LIF => {NPUCorticalAreaIdentifierFlag::InterNeuronLIF(quantization_level)}
            CorticalAreaModelType::Izhikevich => {NPUCorticalAreaIdentifierFlag::InterNeuronIzhikevich(quantization_level)}
            CorticalAreaModelType::HodgkinHuxley => {NPUCorticalAreaIdentifierFlag::InterNeuronHodgkinHuxley(quantization_level)}
        }
    }
    
    pub fn is_dimensional(&self) -> bool {
        match &self {
            NPUCorticalAreaIdentifierFlag::Memory(_) => {false}
            _ => true
        }
    }

    pub fn is_interneuron(&self) -> bool {
        match &self {
            NPUCorticalAreaIdentifierFlag::CoreFeagiStandard(_) => {false}
            NPUCorticalAreaIdentifierFlag::SensoryFeagiStandard(_) => {false}
            NPUCorticalAreaIdentifierFlag::MotorFeagiStandard(_) => {false}
            NPUCorticalAreaIdentifierFlag::Memory(_) => {false}
            _ => true
        }
    }
}

// TODO conversions to and from the Feagi-structure enums