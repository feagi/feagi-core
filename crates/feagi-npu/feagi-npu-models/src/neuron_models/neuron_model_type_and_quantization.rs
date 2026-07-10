use crate::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;

// TODO macroize these

// TODO use array search matching instead

/// Intermediate storage medium for `NeuronModelDescriptor`, can be converted using Into
#[derive(Clone, Copy)]
pub struct NeuronModelPackedDescriptor(u8);

impl From<NeuronModelDescriptor> for NeuronModelPackedDescriptor
{
    fn from(value: NeuronModelDescriptor) -> Self {
        match value {
            NeuronModelDescriptor::FeagiStandard(f) => {
                NeuronModelPackedDescriptor(f as u8)
            },
        }
    }
}


/// A single enum that can be converted to an `NeuronModelPackedDescriptor` to represent a neuron 
/// model type and its membrane potential quantization. First 5 bits are for model (allowing 
/// 32 possible models). 
pub enum NeuronModelDescriptor {
    FeagiStandard(FeagiStandardModelQuantizationLevel),
}

impl NeuronModelDescriptor {
    const MODEL_MASK: u8 = 0b1111_1000;
    const QUANT_MASK: u8 = 0b0000_0111;
}

impl From<NeuronModelPackedDescriptor> for NeuronModelDescriptor {
    fn from(value: NeuronModelPackedDescriptor) -> Self {
        let model = value.0 & Self::MODEL_MASK;
        match model {
            FeagiStandardModelQuantizationLevel::MODEL_BITS => {
                let quant = value.0 & Self::QUANT_MASK;
                
            }
        }
    }
}

impl Default for NeuronModelDescriptor {
    fn default() -> Self {
        NeuronModelDescriptor::FeagiStandard(FeagiStandardModelQuantizationLevel::default())
    }
}
