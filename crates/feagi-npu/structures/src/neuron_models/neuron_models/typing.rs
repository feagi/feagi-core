use crate::neuron_models::neuron_models::feagi_standard::quantization::FeagiStandardModelQuantizationLevel;

// TODO macroize this stuff

pub enum NeuronModelTypeAndQuantizationNested
{
    FeagiStandard(FeagiStandardModelQuantizationLevel)
}

impl Default for NeuronModelTypeAndQuantizationNested {
    fn default() -> Self {
        NeuronModelTypeAndQuantizationNested::FeagiStandard(FeagiStandardModelQuantizationLevel::default())
    }
}


#[repr(u8)]
#[derive(Default)]
pub enum NeuronModelTypeAndQuantizationFlat
{
    #[default]
    FeagiStandard_Standard32Bit = 0
}

impl Into<NeuronModelTypeAndQuantizationNested> for NeuronModelTypeAndQuantizationFlat
{
    fn into(self) -> NeuronModelTypeAndQuantizationNested {
        match self {

            NeuronModelTypeAndQuantizationFlat::FeagiStandard_Standard32Bit => {
                NeuronModelTypeAndQuantizationNested::FeagiStandard(
                    FeagiStandardModelQuantizationLevel::Standard32bit
                )
            }

        }
    }
}

impl Into<NeuronModelTypeAndQuantizationFlat> for NeuronModelTypeAndQuantizationNested
{
    fn into(self) -> NeuronModelTypeAndQuantizationFlat {
        match self {

            NeuronModelTypeAndQuantizationNested::FeagiStandard(quants) => {
                match quants {

                    FeagiStandardModelQuantizationLevel::Standard32bit => {
                        NeuronModelTypeAndQuantizationFlat::FeagiStandard_Standard32Bit
                    }

                }
            }

        }
    }
}