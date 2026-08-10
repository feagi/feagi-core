use crate::cortical_area::components::cortical_area_layout::enums::CorticalAreaLayoutNested;
use crate::cortical_area::components::cortical_area_layout::implementations::dimensional::CorticalAreaLayoutDimensional;
use crate::cortical_area::genome_compose::cortical_writer::NeuronModelCorticalWriter;
use crate::cortical_area::neuron::cortical_area_properties::{CorticalAreaProperties, PostCorticalPotential};
use crate::cortical_area::neuron::neuron_properties::NeuronProperties;
use crate::cortical_area::neuron_model_implementations::feagi_advanced::data::{
    ConsecutiveFireCountdown, ConsecutiveFireLimit, DegeneracyConstant, FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData,
    LeakCoefficient, RefractoryCountdown, RefractoryPeriodLimit, SnoozePeriod,
};
use crate::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelQuantization;
use core::marker::PhantomData;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_data::values::quantizable::{PercentageUnsigned, QuantizedUnsignedIntegerTrait};

#[derive(Debug, Clone, Copy)]
pub enum FeagiAdvancedModelCorticalWriter<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    DefaultNewDimensional {
        dimensions: DimensionalCorticalArea4DDimensions<<FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant>,
        _p: PhantomData<NMQ>,
    },
}

impl<NMQ> NeuronModelCorticalWriter<NMQ, FeagiAdvancedModelCorticalData<NMQ>, FeagiAdvancedModelNeuronData<NMQ>>
    for FeagiAdvancedModelCorticalWriter<NMQ>
where
    NMQ: FeagiAdvancedModelQuantization,
{
    fn number_neurons_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::NeuronIndexQuant, ()> {
        match self {
            FeagiAdvancedModelCorticalWriter::DefaultNewDimensional { dimensions, _p: _ } => {
                let u = dimensions.number_contained_elements();
                let r: FIQ::NeuronIndexQuant = u.try_to_quantization().unwrap(); // TODO error handling!
                Ok(r)
            }
        }
    }

    fn write_to_cortical_area<FIQ: FeagiIndexQuantization>(
        self,
        cortical_data: &mut FeagiAdvancedModelCorticalData<NMQ>,
        neuron_data: &mut [FeagiAdvancedModelNeuronData<NMQ>],
        neuron_properties_out: &mut [NeuronProperties],
    ) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties<NMQ>), ()> {
        match self {
            FeagiAdvancedModelCorticalWriter::DefaultNewDimensional { dimensions, _p } => {
                // TODO these should really be coming from some cortical class definition

                // Uniform
                let new_cortical: FeagiAdvancedModelCorticalData<NMQ> = FeagiAdvancedModelCorticalData {
                    excitability: PercentageUnsigned::ZERO_PERCENT,
                    refractory_period_limit: RefractoryPeriodLimit::QUANT_ONE,
                    fire_threshold_limit: NeuronMembranePotential::QUANT_ONE,
                    consecutive_fire_limit: ConsecutiveFireLimit::QUANT_ONE,
                    snooze_period: SnoozePeriod::QUANT_ONE,
                    degeneracy_constant: DegeneracyConstant::QUANT_ONE,
                };

                let new_cortical_properties = CorticalAreaProperties {
                    post_cortical_potential: PostCorticalPotential::MembraneDriven,
                    probe_cortical_area_input_disabled: false,
                    probe_cortical_area_output_disabled: false,
                    is_psp_uniform: false,
                };

                let new_uniform_neuron: FeagiAdvancedModelNeuronData<NMQ> = FeagiAdvancedModelNeuronData {
                    neuron_fire_threshold: NeuronMembranePotential::QUANT_ONE,
                    neuron_leak_coefficient: LeakCoefficient::QUANT_ONE,
                    neuron_refractory_countdown: RefractoryCountdown::QUANT_ONE,
                    neuron_consecutive_fire_countdown: ConsecutiveFireCountdown::QUANT_ONE,
                };

                let new_uniform_neuron_properties = NeuronProperties {
                    probe_force_disabled: false,
                    probe_force_firing: false,
                };

                // `dimensions` is already in genomic quantization, which is exactly what the
                // genomic-parameterized layout expects, so use it directly (no re-quantization).
                let layout = CorticalAreaLayoutNested::Dimensional(CorticalAreaLayoutDimensional { dimensions });

                *cortical_data = new_cortical;
                neuron_data.fill(new_uniform_neuron);
                neuron_properties_out.fill(new_uniform_neuron_properties);
                Ok((layout, new_cortical_properties))
            }
        }
    }
}
