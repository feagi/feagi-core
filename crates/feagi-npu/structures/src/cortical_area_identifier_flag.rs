use feagi_structures::base_quantizable::QuantizationLevel;


// NOTE: Due to quantization level taking 2 bits automatically, we have 6 bits for the cortical,
// meaning within a u8 we can identify 64 different types of cortical areas

/// Optimally allows categorizing different types of cortical areas and their quantization levels
/// in the space of a single byte
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum NPUCorticalAreaIdentifierFlag {
    Core(QuantizationLevel),
    Sensor(QuantizationLevel),
    Motor(QuantizationLevel),
    FeagiStandard(QuantizationLevel),
    LIF(QuantizationLevel),
    Izhikevich(QuantizationLevel),
    HodgkinHuxley(QuantizationLevel),
    Memory(QuantizationLevel),
}

impl NPUCorticalAreaIdentifierFlag {
    pub fn is_dimensional(&self) -> bool {
        match &self {
            NPUCorticalAreaIdentifierFlag::Memory(_) => {false}
            _ => true
        }
    }

    pub fn is_interneuron(&self) -> bool {
        match &self {
            NPUCorticalAreaIdentifierFlag::Core(_) => {false}
            NPUCorticalAreaIdentifierFlag::Sensor(_) => {false}
            NPUCorticalAreaIdentifierFlag::Motor(_) => {false}
            NPUCorticalAreaIdentifierFlag::Memory(_) => {false}
            _ => true
        }
    }
}

// TODO conversions to and from the Feagi-structure enums