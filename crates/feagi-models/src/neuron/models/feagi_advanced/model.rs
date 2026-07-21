use crate::burst_index::BurstIndex;
use crate::neuron::genome_interface::neuron_model_writer::{NeuronModelWriter, UniformNeuronModelWriter};
use crate::neuron::models::feagi_advanced::data::{
    ConsecutiveFireCountdown, ConsecutiveFireLimit, DegeneracyConstant, FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData,
    LeakCoefficient, RefractoryCountdown, RefractoryPeriodLimit, SnoozePeriod,
};
use crate::neuron::models::feagi_advanced::quantization::FeagiAdvancedModelQuantization;
use crate::neuron::models_shared_traits::model::NeuronModel;
use crate::neuron::common_structs::neuron_history::NeuronHistoryFull;
use core::marker::PhantomData;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::PercentageUnsigned;
use feagi_data::values::quantizable::QuantizedElementBase;

pub struct FeagiAdvancedModel<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiAdvancedModelQuantization, // fsm quant impl
{
    // No actual members
    _p: PhantomData<(FIQ, CPQ)>,
}

impl<FIQ, CPQ> NeuronModel<FIQ, CPQ, FeagiAdvancedModelCorticalData<CPQ>, FeagiAdvancedModelNeuronData<CPQ>> for FeagiAdvancedModel<FIQ, CPQ>
where
    FIQ: FeagiIndexQuantization,
    CPQ: FeagiAdvancedModelQuantization,
{
    const MODEL_NEURON_DATA_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool = true;
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool = true;
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_NONE: bool = false;
    type UsedNeuronHistory = NeuronHistoryFull<FIQ>;

    fn process_neuron_potential_for_dimensional_layout_cortical_area(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::UsedNeuronHistory,
        cortical_area_data: &FeagiAdvancedModelCorticalData<CPQ>,
        neuron_model_data: &mut FeagiAdvancedModelNeuronData<CPQ>,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool {
        // update neuron potential
        *this_neuron_potential += *incoming_potential; // - QuantizedDecimalTrait::QUANT_ZERO )); // TODO right now subtracting 0, but this is the resting potential



        false
    }

    fn process_neuron_potential_for_none_layout_cortical_area(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        memory_cortical_number_neurons: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::UsedNeuronHistory,
        cortical_area_data: &FeagiAdvancedModelCorticalData<CPQ>,
        neuron_model_data: &mut FeagiAdvancedModelNeuronData<CPQ>,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool {
        panic!("not implemented yet");
    }

    fn default_neuron_writer_dimensional_layout_cortical_area(
        _dimensional_cortical_dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) -> Option<impl NeuronModelWriter<CPQ, FeagiAdvancedModelCorticalData<CPQ>, FeagiAdvancedModelNeuronData<CPQ>>> {
        let default_cortical_area: FeagiAdvancedModelCorticalData<CPQ> = FeagiAdvancedModelCorticalData::new(
            PercentageUnsigned::ZERO_PERCENT,
            RefractoryPeriodLimit::QUANT_ZERO,
            NeuronMembranePotential::QUANT_ONE,
            ConsecutiveFireLimit::QUANT_ONE,
            SnoozePeriod::QUANT_ONE,
            DegeneracyConstant::QUANT_ONE,
        );
        let default_neuron: FeagiAdvancedModelNeuronData<CPQ> = FeagiAdvancedModelNeuronData::new(
            NeuronMembranePotential::QUANT_ONE,
            LeakCoefficient::QUANT_ONE,
            RefractoryCountdown::QUANT_ONE,
            ConsecutiveFireCountdown::QUANT_ONE,
        );
        let writer = UniformNeuronModelWriter::new(Some(default_cortical_area), Some(default_neuron));
        Some(writer)
    }

    fn default_neuron_writer_none_layout_cortical_area(
        number_neurons: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) -> Option<impl NeuronModelWriter<CPQ, FeagiAdvancedModelCorticalData<CPQ>, FeagiAdvancedModelNeuronData<CPQ>>> {
        None::<UniformNeuronModelWriter<CPQ, FeagiAdvancedModelCorticalData<CPQ>, FeagiAdvancedModelNeuronData<CPQ>>>
    }
}
